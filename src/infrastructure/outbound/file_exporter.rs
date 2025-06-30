use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::Context;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::domain::{
    model::{entity::balance::Balance, error::ClientError},
    port::outbound::balance_exporter::BalanceExporter,
};

const FILE_EXTENSION: &str = ".DAT";
const DIRECTORY: &str = ".";

pub struct FileExporter {
    counter: AtomicUsize,
}

impl FileExporter {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let mut entries = tokio::fs::read_dir(DIRECTORY).await?;
        let mut last_file_counter = 0;

        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if file_name_str.ends_with(FILE_EXTENSION) {
                if let Some(counter) = extract_counter(&file_name_str) {
                    last_file_counter = last_file_counter.max(counter);
                }
            }
        }

        Ok(Self {
            counter: AtomicUsize::new(last_file_counter),
        })
    }
}

fn extract_counter(file_name: &str) -> Option<usize> {
    file_name
        .split('_')
        .nth(1)?
        .split('.')
        .next()?
        .parse::<usize>()
        .ok()
}

impl BalanceExporter for FileExporter {
    /// Exports the balances to a file with the format "DDMMYYYY_COUNTER.DAT"
    /// where DDMMYYYY is the current date and COUNTER is a counter that is incremented for each file.
    ///
    /// # Arguments
    ///
    /// * `balances` - The balances to export. It is expected to be non-empty. If it is empty, the function returns an error.
    ///
    /// # Errors
    ///
    /// - [ClientError::BalancesEmpty] if the balances are empty.
    /// - [ClientError::Unknown] if the balances cannot be exported.
    async fn export_balances(&self, balances: &[Balance]) -> Result<(), ClientError> {
        if balances.is_empty() {
            return Err(ClientError::BalancesEmpty);
        }

        let counter = self.counter.fetch_add(1, Ordering::Relaxed) + 1;

        let file_name = format!("{}_{}.DAT", chrono::Utc::now().format("%d%m%Y"), counter);

        let file_path = format!("{DIRECTORY}/{file_name}");
        let mut file = File::create(&file_path)
            .await
            .with_context(|| format!("Error creating file: {file_path}"))?;

        for balance in balances {
            file.write_all(format!("{} {}\n", balance.client_id(), balance.balance()).as_bytes())
                .await
                .with_context(|| format!("Error writing to file: {file_path}"))?;
        }

        Ok(())
    }
}
