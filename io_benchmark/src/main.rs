use bytesize::ByteSize;
use clap::Parser;
use rand::Rng;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

// ANSI Colors for premium visual styling
const COLOR_RESET: &str = "\x1b[0m";
const COLOR_BOLD: &str = "\x1b[1m";
const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_CYAN: &str = "\x1b[36m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_RED: &str = "\x1b[31m";

#[derive(Parser, Debug)]
#[command(name = "Disk I/O Benchmark")]
#[command(about = "A premium console-based I/O benchmarking tool comparing sequential vs random reads.", long_about = None)]
struct Config {
    #[arg(long = "path", default_value = "./benchmark_test.bin", help = "Path to the test file")]
    file_path: PathBuf,

    #[arg(long = "size", default_value = "128M", value_parser = parse_size_with_bytesize, help = "Size of the test file (e.g. 64M, 256M, 1G)")]
    file_size: ByteSize,

    #[arg(long = "block-size", default_value = "4K", value_parser = parse_size_with_bytesize, help = "Block size for reads (e.g. 4K, 64K, 1M)")]
    block_size: ByteSize,

    #[arg(long = "duration", default_value = "5s", value_parser = humantime::parse_duration, help = "Duration to run random read test (e.g. 5s, 2m)")]
    duration: Duration,

    #[arg(long = "ops", help = "Exact number of operations for random read test (overrides duration if set)")]
    ops: Option<u64>,

    #[arg(long = "nocache", help = "Bypass the OS Page Cache (using F_NOCACHE / O_DIRECT)")]
    nocache: bool,

    #[arg(long = "keep", help = "Keep the test file after benchmark completes")]
    keep_file: bool,
}

fn parse_size_with_bytesize(s: &str) -> Result<ByteSize, String> {
    let s = s.trim().to_uppercase();
    if s.is_empty() {
        return Err("Empty size string".to_string());
    }

    // Normalize all single-letter and standard KB/MB/GB suffixes to binary KiB/MiB/GiB
    let normalized = if s.ends_with("KIB") || s.ends_with("MIB") || s.ends_with("GIB") || s.ends_with("TIB") {
        s
    } else if s.ends_with("KB") {
        format!("{}IB", &s[..s.len() - 1]) // KB -> KIB
    } else if s.ends_with("MB") {
        format!("{}IB", &s[..s.len() - 1]) // MB -> MIB
    } else if s.ends_with("GB") {
        format!("{}IB", &s[..s.len() - 1]) // GB -> GIB
    } else if s.ends_with("TB") {
        format!("{}IB", &s[..s.len() - 1]) // TB -> TIB
    } else if s.ends_with('K') {
        format!("{}IB", s) // K -> KIB
    } else if s.ends_with('M') {
        format!("{}IB", s) // M -> MIB
    } else if s.ends_with('G') {
        format!("{}IB", s) // G -> GIB
    } else if s.ends_with('T') {
        format!("{}IB", s) // T -> TIB
    } else {
        s
    };

    normalized.parse::<ByteSize>().map_err(|e| e.to_string())
}

fn format_size(bytes: u64) -> String {
    ByteSize::b(bytes).to_string_as(true)
}

fn format_throughput(bytes_per_sec: f64) -> String {
    format!("{}/s", ByteSize::b(bytes_per_sec as u64).to_string_as(true))
}

fn format_duration(d: Duration) -> String {
    humantime::format_duration(d).to_string()
}

#[cfg(target_os = "macos")]
fn disable_cache(file: &File) -> Result<(), anyhow::Error> {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    let res = unsafe { libc::fcntl(fd, libc::F_NOCACHE, 1) };
    if res == -1 {
        Err(anyhow::anyhow!(
            "Failed to set F_NOCACHE: {}",
            std::io::Error::last_os_error()
        ))
    } else {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn disable_cache(file: &File) -> Result<(), anyhow::Error> {
    use std::os::unix::io::AsRawFd;
    let fd = file.as_raw_fd();
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags == -1 {
        return Err(anyhow::anyhow!("Failed to get file flags (F_GETFL)"));
    }
    let res = unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_DIRECT) };
    if res == -1 {
        Err(anyhow::anyhow!(
            "Failed to set O_DIRECT: {}",
            std::io::Error::last_os_error()
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn disable_cache(_file: &File) -> Result<(), anyhow::Error> {
    Err(anyhow::anyhow!(
        "Bypassing page cache is not implemented on this operating system"
    ))
}

fn parse_args() -> Result<Config, anyhow::Error> {
    let config = Config::parse();

    if config.block_size.as_u64() == 0 {
        return Err(anyhow::anyhow!("Block size cannot be zero"));
    }
    if config.file_size.as_u64() == 0 {
        return Err(anyhow::anyhow!("File size cannot be zero"));
    }
    if config.file_size.as_u64() < config.block_size.as_u64() {
        return Err(anyhow::anyhow!(
            "File size must be at least as large as the block size"
        ));
    }

    Ok(config)
}

fn create_test_file(config: &Config) -> Result<(), anyhow::Error> {
    println!(
        "\n{}[1/3] Preparing test file...{}{}",
        COLOR_BOLD, COLOR_CYAN, COLOR_RESET
    );
    println!("Path: {}", config.file_path.display());
    println!(
        "Size: {} ({})",
        format_size(config.file_size.as_u64()),
        config.file_size.as_u64()
    );

    let start = Instant::now();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&config.file_path)?;

    // We write random/dummy data in chunks to create the file and test write speeds
    let chunk_size = 1024 * 1024; // 1 MB write chunks
    let mut buffer = vec![0u8; chunk_size];
    // Fill the buffer with dummy non-zero patterns so the OS or FS doesn't optimize it as a sparse file
    for (j, item) in buffer.iter_mut().enumerate() {
        *item = (j % 251) as u8;
    }

    let mut remaining = config.file_size.as_u64();
    let mut total_written = 0u64;

    while remaining > 0 {
        let to_write = std::cmp::min(remaining, chunk_size as u64) as usize;
        file.write_all(&buffer[..to_write])?;
        remaining -= to_write as u64;
        total_written += to_write as u64;

        // Print progress
        let percent = (total_written as f64 / config.file_size.as_u64() as f64) * 100.0;
        print!("\rWriting data: {:.1}% completed...", percent);
        let _ = std::io::stdout().flush();
    }

    file.sync_all()?;
    let elapsed = start.elapsed();
    let write_throughput = total_written as f64 / elapsed.as_secs_f64();

    println!(
        "\rWriting data: {}Completed! (Time: {}, Write Speed: {}){}",
        COLOR_GREEN,
        format_duration(elapsed),
        format_throughput(write_throughput),
        COLOR_RESET
    );

    Ok(())
}

fn run_sequential_benchmark(config: &Config) -> Result<(u64, Duration, f64, f64), anyhow::Error> {
    println!(
        "\n{}[2/3] Running Sequential Reads Benchmark...{}{}",
        COLOR_BOLD, COLOR_CYAN, COLOR_RESET
    );
    println!("Block size: {}", format_size(config.block_size.as_u64()));

    let mut file = File::open(&config.file_path)?;
    if config.nocache {
        match disable_cache(&file) {
            Ok(_) => println!(
                "{}Direct I/O Enabled: OS Page Cache bypassed.{}",
                COLOR_YELLOW, COLOR_RESET
            ),
            Err(e) => println!(
                "{}Warning: Could not bypass OS Page Cache: {}{}",
                COLOR_RED, e, COLOR_RESET
            ),
        }
    }

    let mut buffer = vec![0u8; config.block_size.as_u64() as usize];
    let start = Instant::now();
    let mut bytes_read = 0u64;
    let mut ios = 0u64;

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bytes_read += n as u64;
        ios += 1;
    }

    let elapsed = start.elapsed();
    let seconds = elapsed.as_secs_f64();
    let throughput = bytes_read as f64 / seconds;
    let iops = ios as f64 / seconds;

    println!("Total Bytes Read: {}", format_size(bytes_read));
    println!("Elapsed Time:     {}", format_duration(elapsed));
    println!(
        "Throughput:       {}{}{}",
        COLOR_BOLD,
        format_throughput(throughput),
        COLOR_RESET
    );
    println!("IOPS:             {}{:.2}{}", COLOR_BOLD, iops, COLOR_RESET);

    Ok((bytes_read, elapsed, throughput, iops))
}

fn run_random_benchmark(
    config: &Config,
) -> Result<(u64, Duration, f64, f64, Duration), anyhow::Error> {
    println!(
        "\n{}[3/3] Running Random Reads Benchmark...{}{}",
        COLOR_BOLD, COLOR_CYAN, COLOR_RESET
    );
    println!("Block size: {}", format_size(config.block_size.as_u64()));

    let mut file = File::open(&config.file_path)?;
    if config.nocache {
        match disable_cache(&file) {
            Ok(_) => println!(
                "{}Direct I/O Enabled: OS Page Cache bypassed.{}",
                COLOR_YELLOW, COLOR_RESET
            ),
            Err(e) => println!(
                "{}Warning: Could not bypass OS Page Cache: {}{}",
                COLOR_RED, e, COLOR_RESET
            ),
        }
    }

    let block_size_u64 = config.block_size.as_u64();
    let num_blocks = config.file_size.as_u64() / block_size_u64;
    if num_blocks == 0 {
        return Err(anyhow::anyhow!(
            "File is too small for the selected block size"
        ));
    }

    let mut buffer = vec![0u8; config.block_size.as_u64() as usize];
    let mut rng = rand::thread_rng();

    let mut bytes_read = 0u64;
    let mut ios = 0u64;
    let start = Instant::now();

    match config.ops {
        Some(target_ops) => {
            println!("Target I/O Operations: {}", target_ops);
            for _ in 0..target_ops {
                let block_index = rng.gen_range(0..num_blocks);
                let offset = block_index * block_size_u64;
                file.seek(SeekFrom::Start(offset))?;
                file.read_exact(&mut buffer)?;
                bytes_read += block_size_u64;
                ios += 1;
            }
        }
        None => {
            println!("Target Duration:      {}", format_duration(config.duration));
            while start.elapsed() < config.duration {
                let block_index = rng.gen_range(0..num_blocks);
                let offset = block_index * block_size_u64;
                file.seek(SeekFrom::Start(offset))?;
                file.read_exact(&mut buffer)?;
                bytes_read += block_size_u64;
                ios += 1;
            }
        }
    }

    let elapsed = start.elapsed();
    let seconds = elapsed.as_secs_f64();
    let throughput = bytes_read as f64 / seconds;
    let iops = ios as f64 / seconds;
    let avg_latency = if ios > 0 {
        elapsed / ios as u32
    } else {
        Duration::ZERO
    };

    println!("Total Operations: {}", ios);
    println!("Total Bytes Read: {}", format_size(bytes_read));
    println!("Elapsed Time:     {}", format_duration(elapsed));
    println!(
        "Throughput:       {}{}{}",
        COLOR_BOLD,
        format_throughput(throughput),
        COLOR_RESET
    );
    println!("IOPS:             {}{:.2}{}", COLOR_BOLD, iops, COLOR_RESET);
    println!(
        "Average Latency:  {}{}{}",
        COLOR_BOLD,
        format_duration(avg_latency),
        COLOR_RESET
    );

    Ok((bytes_read, elapsed, throughput, iops, avg_latency))
}

fn main() -> Result<(), anyhow::Error> {
    let config = parse_args()?;

    println!("============================================================");
    println!(
        "{}                 DISK I/O BENCHMARK SUITE                 {}",
        COLOR_BOLD, COLOR_RESET
    );
    println!("============================================================");
    println!("Configuration:");
    println!("  Test File:     {}", config.file_path.display());
    println!("  File Size:     {}", format_size(config.file_size.as_u64()));
    println!("  Block Size:    {}", format_size(config.block_size.as_u64()));
    println!(
        "  Bypass Cache:  {}",
        if config.nocache { "Yes" } else { "No" }
    );
    println!("============================================================");

    create_test_file(&config)?;

    let seq_res = run_sequential_benchmark(&config);
    let rand_res = run_random_benchmark(&config);

    // Clean up
    if !config.keep_file && config.file_path.exists() {
        println!("\nCleaning up test file...");
        std::fs::remove_file(&config.file_path)?;
        println!("Cleanup complete!");
    }

    println!("\n============================================================");
    println!(
        "{}                      SUMMARY OF RESULTS                    {}",
        COLOR_BOLD, COLOR_RESET
    );
    println!("============================================================");

    // Print side-by-side comparison
    if let (
        Ok((seq_bytes, seq_time, seq_tp, seq_iops)),
        Ok((rand_bytes, rand_time, rand_tp, rand_iops, rand_lat)),
    ) = (&seq_res, &rand_res)
    {
        println!(
            "{:<20} | {:<20} | {:<20}",
            "Metric", "Sequential Reads", "Random Reads"
        );
        println!("------------------------------------------------------------");
        println!(
            "{:<20} | {:<20} | {:<20}",
            "Bytes Read",
            format_size(*seq_bytes),
            format_size(*rand_bytes)
        );
        println!(
            "{:<20} | {:<20} | {:<20}",
            "Duration",
            format_duration(*seq_time),
            format_duration(*rand_time)
        );
        println!(
            "{:<20} | {}{:<18}{} | {}{:<18}{}",
            "Throughput",
            COLOR_GREEN,
            format_throughput(*seq_tp),
            COLOR_RESET,
            COLOR_GREEN,
            format_throughput(*rand_tp),
            COLOR_RESET
        );
        println!(
            "{:<20} | {}{:<18.2}{} | {}{:<18.2}{}",
            "IOPS", COLOR_CYAN, seq_iops, COLOR_RESET, COLOR_CYAN, rand_iops, COLOR_RESET
        );
        println!(
            "{:<20} | {:<20} | {}{:<18}{}",
            "Avg Latency",
            "N/A",
            COLOR_YELLOW,
            format_duration(*rand_lat),
            COLOR_RESET
        );

        let ratio = if *rand_tp > 0.0 {
            seq_tp / rand_tp
        } else {
            0.0
        };
        println!("------------------------------------------------------------");
        println!(
            "Sequential reads were {}{:.1}x{} faster than random reads.",
            COLOR_BOLD, ratio, COLOR_RESET
        );
    } else {
        println!("Benchmark failed to complete successfully.");
    }
    println!("============================================================\n");

    // Propagate errors if any
    seq_res?;
    rand_res?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_with_bytesize() {
        assert_eq!(parse_size_with_bytesize("4K").unwrap().as_u64(), 4096);
        assert_eq!(parse_size_with_bytesize("1M").unwrap().as_u64(), 1024 * 1024);
        assert_eq!(parse_size_with_bytesize("2G").unwrap().as_u64(), 2 * 1024 * 1024 * 1024);
        assert_eq!(parse_size_with_bytesize(" 128 MB ").unwrap().as_u64(), 128 * 1024 * 1024);
        assert!(parse_size_with_bytesize("abc").is_err());
        assert!(parse_size_with_bytesize("").is_err());
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.0 kiB");
        assert_eq!(format_size(1024 * 1024 * 5), "5.0 MiB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 3), "3.0 GiB");
    }

    #[test]
    fn test_format_throughput() {
        assert_eq!(format_throughput(500.0), "500 B/s");
        assert_eq!(format_throughput(1024.0 * 2.5), "2.5 kiB/s");
        assert_eq!(format_throughput(1024.0 * 1024.0 * 123.0), "123.0 MiB/s");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(120)), "2m");
        assert_eq!(format_duration(Duration::from_secs(5)), "5s");
        assert_eq!(format_duration(Duration::from_millis(250)), "250ms");
        assert_eq!(format_duration(Duration::from_micros(15)), "15us");
    }
}
