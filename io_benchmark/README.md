# Disk I/O Benchmark Suite

A utility written in Rust to benchmark and compare sequential vs. random reads on storage media.

## Usage

### Build and Run with Bazel
To execute the benchmark suite hermetically through Bazel:
```bash
# Default balanced run (128MiB file size, 4KiB block size, caching enabled)
bazel run //io_benchmark

# Bypass page cache (Direct I/O)
bazel run //io_benchmark -- --nocache

# Run a custom 2-second loop for both read patterns
bazel run //io_benchmark -- --size 16MiB --block-size 4KiB --duration 2s --nocache

# Custom size and block size
bazel run //io_benchmark -- --size 256MiB --block-size 64KiB --nocache
```

### CLI Parameters
- `--path <path>`: Path to write the temporary benchmark file (defaults to `./benchmark_test.bin`).
- `--size <size>`: Size of the test file (defaults to `128MiB`). Supports binary suffixes: `KiB`, `MiB`, `GiB`, `TiB`.
- `--block-size <block>`: Read buffer block size (defaults to `4KiB`).
- `--duration <duration>`: If set, loops read operations for the specified duration (e.g. `5s`, `2m`).
- `--ops <count>`: If set, executes exactly `<count>` read operations.
- `--nocache`: Bypass the OS Page Cache.
- `--keep`: Keep the temporary file after run completion.

---

## Experimental Results

The following benchmarks were conducted on a macOS host machine with a solid-state drive (SSD) using a **128.0 MiB** file size.

### Experiment 1: Cached I/O (4 KiB Blocks)
*Command: `bazel run //io_benchmark -- --size 128MiB --block-size 4KiB`*

| Metric          | Sequential Reads | Random Reads      | Comparison                |
| --------------- | ---------------- | ----------------- | ------------------------- |
| **Bytes Read**  | 128.0 MiB        | 128.0 MiB         | Equal                     |
| **Duration**    | 21ms 328us 250ns | 122ms 915us 959ns | Sequential is 5.7x faster |
| **Throughput**  | 5.9 GiB/s        | 1.0 GiB/s         | -                         |
| **IOPS**        | 1536366.09       | 266588.65         | -                         |
| **Avg Latency** | N/A              | 3us 751ns         | -                         |

---

### Experiment 2: Direct I/O - Page Cache Bypassed (4 KiB Blocks)
*Command: `bazel run //io_benchmark -- --size 128MiB --block-size 4KiB --nocache`*

| Metric          | Sequential Reads | Random Reads     | Comparison                |
| --------------- | ---------------- | ---------------- | ------------------------- |
| **Bytes Read**  | 128.0 MiB        | 128.0 MiB        | Equal                     |
| **Duration**    | 24ms 104us 167ns | 70ms 635us 125ns | Sequential is 2.9x faster |
| **Throughput**  | 5.2 GiB/s        | 1.8 GiB/s        | -                         |
| **IOPS**        | 1359433.00       | 463905.17        | -                         |
| **Avg Latency** | N/A              | 2us 155ns        | -                         |

---

### Experiment 3: Large Blocks Direct I/O (1 MiB Blocks)
*Command: `bazel run //io_benchmark -- --size 128MiB --block-size 1MiB --nocache`*

| Metric          | Sequential Reads | Random Reads    | Comparison |
| --------------- | ---------------- | --------------- | ---------- |
| **Bytes Read**  | 128.0 MiB        | 128.0 MiB       | Equal      |
| **Duration**    | 10ms 515us 250ns | 9ms 995us 292ns | 1:1 Parity |
| **Throughput**  | 11.9 GiB/s       | 12.5 GiB/s      | -          |
| **IOPS**        | 12172.80         | 12806.03        | -          |
| **Avg Latency** | N/A              | 78us 88ns       | -          |

---

## Performance Analysis
1. **Access Pattern Overhead**: With small block sizes (4 KiB), sequential reads outpace random reads by **2.9x** to **5.7x** due to sequential block prefetching mechanisms inside SSD controller firmware and the OS kernel.
2. **Page Cache Impact**: Bypassing the page cache reduces peak sequential read performance slightly (from `5.9 GiB/s` to `5.2 GiB/s`), but allows measuring raw SSD throughput limits accurately without system memory caching bias.
3. **Block Size Tuning**: Moving to a large block size (1 MiB) fully saturates the hardware interface, increasing throughput to **11.9+ GiB/s** for both sequential and random reads, highlighting that SSD seek latency overhead becomes negligible when processing large contiguous blocks of data.
