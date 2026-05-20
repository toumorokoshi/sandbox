use anyhow::Result;
use arrow::array::{Int64Array, RecordBatch};
use arrow::datatypes::{DataType, Field, Schema};
use futures::stream::StreamExt;
use lance::dataset::{Dataset, WriteParams};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::arrow_writer::ArrowWriter;
use rand::RngCore;
use std::fs::File;
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting format benchmark (Parquet vs Lance)");

    // Test for different payload sizes (representing different column sizes)
    // 1 KB, 10 KB, 100 KB, 1 MB, 10 MB
    let sizes = vec![1024, 10 * 1024, 100 * 1024, 1024 * 1024, 10 * 1024 * 1024];
    let total_data_size = 100 * 1024 * 1024; // 100 MB of data total per benchmark run

    println!(
        "{:<15} | {:<20} | {:<20}",
        "Payload Size", "Parquet Read (MB/s)", "Lance Read (MB/s)"
    );
    println!("{:-<15}-+-{:-<20}-+-{:-<20}-", "", "", "");

    for size in sizes {
        let rows = total_data_size / size;
        let rows = if rows == 0 { 1 } else { rows };

        let batch = generate_batch(rows, size)?;

        let parquet_path = format!("/tmp/bench_{}.parquet", size);
        let lance_path = format!("/tmp/bench_{}.lance", size);

        // Clean up previous runs if any
        let _ = std::fs::remove_file(&parquet_path);
        let _ = std::fs::remove_dir_all(&lance_path);

        // Write Parquet
        let file = File::create(&parquet_path)?;
        let mut writer = ArrowWriter::try_new(file, batch.schema(), None)?;
        writer.write(&batch)?;
        writer.close()?;

        // Write Lance
        let reader =
            arrow::record_batch::RecordBatchIterator::new(vec![Ok(batch.clone())], batch.schema());
        let _ = Dataset::write(reader, &lance_path, Some(WriteParams::default())).await?;

        // Read Parquet
        let start = Instant::now();
        let p_file = File::open(&parquet_path)?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(p_file)?;
        let reader = builder.build()?;
        let mut p_rows = 0;
        for rb in reader {
            let rb = rb?;
            p_rows += rb.num_rows();
        }
        let p_duration = start.elapsed();
        let p_throughput = (total_data_size as f64 / 1024.0 / 1024.0) / p_duration.as_secs_f64();

        // Read Lance
        let start = Instant::now();
        let dataset = Dataset::open(&lance_path).await?;
        let scanner = dataset.scan();
        let mut stream = scanner.try_into_stream().await?;
        let mut l_rows = 0;
        while let Some(rb) = stream.next().await {
            let rb = rb?;
            l_rows += rb.num_rows();
        }
        let l_duration = start.elapsed();
        let l_throughput = (total_data_size as f64 / 1024.0 / 1024.0) / l_duration.as_secs_f64();

        assert_eq!(p_rows, rows);
        assert_eq!(l_rows, rows);

        let size_str = bytesize::ByteSize::b(size as u64).to_string();
        println!(
            "{:<15} | {:<20.2} | {:<20.2}",
            size_str, p_throughput, l_throughput
        );

        // Clean up
        let _ = std::fs::remove_file(&parquet_path);
        let _ = std::fs::remove_dir_all(&lance_path);
    }

    Ok(())
}

fn generate_batch(rows: usize, payload_size: usize) -> Result<RecordBatch> {
    let mut rng = rand::thread_rng();

    let mut ids = Vec::with_capacity(rows);
    let mut builder = arrow::array::BinaryBuilder::with_capacity(rows, rows * payload_size);

    let mut buffer = vec![0u8; payload_size];
    for i in 0..rows {
        ids.push(i as i64);
        rng.fill_bytes(&mut buffer);
        builder.append_value(&buffer);
    }

    let id_array = Int64Array::from(ids);
    let payload_array = builder.finish();

    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("payload", DataType::Binary, false),
    ]));

    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![Arc::new(id_array), Arc::new(payload_array)],
    )?;

    Ok(batch)
}
