### Step 1: Set Up Your Rust Project

First, create a new Rust project using Cargo:

```bash
cargo new transaction_viewer
cd transaction_viewer
```

### Step 2: Add Dependencies

Open `Cargo.toml` and add the necessary dependencies:

```toml
[dependencies]
csv = "1.1"
ratatui = "0.5"  # Check for the latest version
tokio = { version = "1", features = ["full"] }
```

### Step 3: Create the Data Structures

Create a new file `src/models.rs` to define the data structures for transactions.

```rust
// src/models.rs

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub amount: f64,
    pub category: String,
}

pub type Transactions = Vec<Transaction>;

pub fn categorize_transactions(transactions: &Transactions) -> HashMap<String, Transactions> {
    let mut categorized: HashMap<String, Transactions> = HashMap::new();

    for transaction in transactions {
        categorized
            .entry(transaction.category.clone())
            .or_insert_with(Vec::new)
            .push(transaction.clone());
    }

    categorized
}
```

### Step 4: Read the CSV File

Create a new file `src/csv_reader.rs` to handle reading the CSV file.

```rust
// src/csv_reader.rs

use crate::models::{Transaction, Transactions};
use csv::ReaderBuilder;
use std::error::Error;

pub fn read_transactions_from_csv(file_path: &str) -> Result<Transactions, Box<dyn Error>> {
    let mut rdr = ReaderBuilder::new().has_headers(true).from_path(file_path)?;
    let mut transactions: Transactions = Vec::new();

    for result in rdr.deserialize() {
        let transaction: Transaction = result?;
        transactions.push(transaction);
    }

    Ok(transactions)
}
```

### Step 5: Create the CLI View

Create a new file `src/view.rs` to handle displaying the transactions using Ratatui.

```rust
// src/view.rs

use crate::models::{categorize_transactions, Transactions};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use std::error::Error;

pub fn display_transactions(transactions: &Transactions) -> Result<(), Box<dyn Error>> {
    let categorized = categorize_transactions(transactions);
    let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;

    terminal.draw(|f| {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(
                vec![Constraint::Percentage(100)].as_ref(),
            )
            .split(size);

        for (category, trans) in categorized {
            let items: Vec<ListItem> = trans
                .iter()
                .map(|t| ListItem::new(format!("ID: {}, Amount: {}", t.id, t.amount)))
                .collect();

            let list = List::new(items)
                .block(Block::default().title(category).borders(Borders::ALL));
            f.render_widget(list, chunks[0]);
        }
    })?;

    Ok(())
}
```

### Step 6: Main Application Logic

Now, modify `src/main.rs` to tie everything together.

```rust
// src/main.rs

mod csv_reader;
mod models;
mod view;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let file_path = "transactions.csv"; // Change this to your CSV file path
    let transactions = csv_reader::read_transactions_from_csv(file_path)?;

    view::display_transactions(&transactions)?;

    Ok(())
}
```

### Step 7: Create a Sample CSV File

Create a sample CSV file named `transactions.csv` in the root of your project:

```csv
id,amount,category
1,100.0,Food
2,200.0,Transport
3,150.0,Food
4,50.0,Entertainment
5,300.0,Transport
```

### Step 8: Run the Application

Now you can run your application:

```bash
cargo run
```

### Conclusion

This modular Rust application reads transaction data from a CSV file, categorizes the transactions, and displays them in a CLI interface using the Ratatui library. You can expand upon this by adding features such as filtering, sorting, or more complex categorization logic.