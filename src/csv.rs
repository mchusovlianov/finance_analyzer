use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct RawTransaction {
    #[serde(rename = "Date")]
    date: String,
    #[serde(rename = "Name / Description")]
    description: String,
    #[serde(rename = "Account")]
    account: String,
    #[serde(rename = "Counterparty")]
    counterparty: String,
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Debit/credit")]
    debit_credit: String,
    #[serde(rename = "Amount (EUR)")]
    amount: String,
    #[serde(rename = "Transaction type")]
    transaction_type: String,
    #[serde(rename = "Notifications")]
    notifications: String,
    #[serde(rename = "Resulting balance")]
    resulting_balance: String,
    #[serde(rename = "Tag")]
    tag: String,
}

pub fn read_transactions_from_csv<P: AsRef<Path>>(path: P) -> Result<Vec<crate::Transaction>> {
    let mut transactions = Vec::new();
    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .flexible(true)
        .from_path(path)
        .context("Failed to open CSV file")?;

    for (index, result) in reader.deserialize::<RawTransaction>().enumerate() {
        match result {
            Ok(raw) => {
                // Parse YYYYMMDD format and set time to midnight
                let date = NaiveDateTime::parse_from_str(&format!("{} 00:00:00", raw.date), "%Y%m%d %H:%M:%S")
                    .with_context(|| format!("Failed to parse date '{}' on line {}", raw.date, index + 2))?;
                
                let mut amount = raw.amount
                    .trim()
                    .replace(',', ".")
                    .parse::<Decimal>()
                    .with_context(|| format!("Failed to parse amount on line {}", index + 2))?;

                // Convert to negative if it's a debit transaction
                if raw.debit_credit == "Debit" {
                    amount = -amount;
                }

                transactions.push(crate::Transaction {
                    id: index as u64,
                    date,
                    amount,
                    merchant: raw.description,
                    description: raw.notifications,
                    category: None,
                });
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse line {}: {}", index + 2, e);
                continue;
            }
        }
    }

    Ok(transactions)
}