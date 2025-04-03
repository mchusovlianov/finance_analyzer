pub mod models;
pub mod ui;
pub mod utils;

// Re-export commonly used items
pub use models::transaction::Transaction;
pub use models::category::{Category, CategoryType};
pub use ui::app::App;