use std::fs::File;
use std::str::FromStr;
use anyhow::Result;
use csv::ReaderBuilder;
use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use crate::models::transaction::Transaction;

fn parse_amount(amount: &str, debit_credit: &str) -> Result<Decimal> {
    let amount = amount.replace(',', ".");
    let mut decimal = Decimal::from_str(&amount)?;
    if debit_credit == "Debit" {
        decimal = -decimal;
    }
    Ok(decimal)
}

fn parse_date(date: &str) -> Result<NaiveDateTime> {
    let date = chrono::NaiveDateTime::parse_from_str(&format!("{}000000", date), "%Y%m%d%H%M%S")?;
    Ok(date)
}

pub fn read_transactions_from_csv(path: &str) -> Result<Vec<Transaction>> {
    let file = File::open(path)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_reader(file);

    let mut id_counter = 1u64;
    let mut transactions = Vec::new();
    
    for result in rdr.records() {
        let record = result?;
        if record.len() < 7 {
            continue;
        }

        let date = parse_date(&record[0].trim_matches('"'))?;
        let merchant = record[1].trim_matches('"').to_string();
        let description = record[8].trim_matches('"').to_string();
        let amount = parse_amount(
            &record[6].trim_matches('"'),
            &record[5].trim_matches('"')
        )?;

        transactions.push(Transaction {
            id: id_counter,
            date,
            amount,
            merchant,
            description,
            category: None,
        });
        id_counter += 1;
    }

    Ok(transactions)
}