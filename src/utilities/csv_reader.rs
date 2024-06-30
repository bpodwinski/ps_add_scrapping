use anyhow::Result;
use csv_async::{AsyncReader, AsyncReaderBuilder};
use tokio::fs::File as AsyncFile;
use tokio::io::BufReader;

/// Sets up CSV file reading.
///
/// # Arguments
/// * `source_path` - The path to the source CSV file.
///
/// # Returns a Result containing the CSV reader if successful, or an error if not.
async fn setup_csv_reader(source_path: &str) -> Result<AsyncReader<BufReader<AsyncFile>>> {
    let file = AsyncFile::open(source_path).await?;
    let reader = BufReader::new(file);
    let csv_reader = AsyncReaderBuilder::new().create_reader(reader);

    Ok(csv_reader)
}
