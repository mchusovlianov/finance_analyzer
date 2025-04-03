use std::collections::HashMap;
use rust_decimal::Decimal;
use ratatui::widgets::ListState;
use crossterm::event::KeyCode;
use crate::models::{
    category::{Category, CategoryType},
    transaction::Transaction,
};

#[derive(Debug)]
pub enum View {
    TransactionList,
    CategorySummary,
    TransactionDetail,
    CategoryDetail,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SortField {
    Date,
    Amount,
    Merchant,
    Category,
}

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Filtering,
    Categorizing,
}

#[derive(Debug)]
pub struct App {
    pub transactions: Vec<Transaction>,
    pub filtered_transactions: Vec<usize>,
    pub categories: HashMap<String, Category>,
    pub current_view: View,
    pub selected_transaction: Option<usize>,
    pub category_totals: HashMap<String, Decimal>,
    pub list_state: ListState,
    pub sort_field: SortField,
    pub sort_order: SortOrder,
    pub input_mode: InputMode,
    pub input_text: String,
    pub filter: Option<String>,
    pub can_show_details: bool,
    pub category_selection: Option<usize>,
    pub available_categories: Vec<CategoryType>,
}

impl App {
    pub fn new(csv_path: &str) -> anyhow::Result<Self> {
        let transactions = crate::utils::csv::read_transactions_from_csv(csv_path)?;
        let mut list_state = ListState::default();
        if !transactions.is_empty() {
            list_state.select(Some(0));
        }

        // Define categories with their rules
        let categories = vec![
            Category::new("Groceries", &[
                ("Albert Heijn", 1),
                ("Picnic", 1),
                ("Crisp", 1),
                ("WILLYS", 1),
                ("Flink", 1),
            ]),
            Category::new("Utilities", &[
                ("ESSENT", 1),
                ("ANWB Energie", 1),
                ("Waternet", 1),
                ("KPN", 1),
            ]),
            Category::new("Transportation", &[
                ("Uber", 1),
                ("TLS BV inz. OV-Chipkaart", 1),
            ]),
            Category::new("Childcare", &[
                ("KINDERGARDEN", 1),
                ("Babysitting", 1),
            ]),
            Category::new("Entertainment", &[
                ("SWESHOP", 1),
                ("Espresso House", 1),
                ("Babbel", 1),
                ("hunkemoller", 1),
            ]),
            Category::new("Government", &[
                ("BELASTINGDIENST", 1),
                ("Gemeente Amsterdam", 1),
            ]),
            Category::new("Internal Transfer", &[
                ("Oranje Spaarrekening", 1),
                ("Hr MA Chusovlyanov", 1),
                ("Mw TI Chusovlyanova", 1),
            ]),
        ];

        let mut app = App {
            transactions,
            filtered_transactions: Vec::new(),
            categories: categories.into_iter().map(|c| (c.name.clone(), c)).collect(),
            current_view: View::TransactionList,
            selected_transaction: None,
            category_totals: HashMap::new(),
            list_state,
            sort_field: SortField::Date,
            sort_order: SortOrder::Descending,
            input_mode: InputMode::Normal,
            input_text: String::new(),
            filter: None,
            can_show_details: false,
            category_selection: None,
            available_categories: CategoryType::all(),
        };

        app.categorize_all_transactions();
        app.update_category_totals();

        Ok(app)
    }

    pub fn next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.transactions.len().saturating_sub(1) {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_transaction = Some(i);
    }

    pub fn previous(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.transactions.len().saturating_sub(1)
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_transaction = Some(i);
    }

    pub fn sort_transactions(&mut self) {
        let sort_field = self.sort_field.clone();
        let sort_order = self.sort_order.clone();
        
        if !self.filtered_transactions.is_empty() {
            let transactions = &self.transactions;
            self.filtered_transactions.sort_by(|&a, &b| {
                let ta = &transactions[a];
                let tb = &transactions[b];
                compare_transactions(ta, tb, &sort_field, &sort_order)
            });
        } else {
            self.transactions.sort_by(|a, b| {
                compare_transactions(a, b, &sort_field, &sort_order)
            });
        }
    }
}

fn compare_transactions(a: &Transaction, b: &Transaction, field: &SortField, order: &SortOrder) -> std::cmp::Ordering {
    let ordering = match field {
        SortField::Date => a.date.cmp(&b.date),
        SortField::Amount => a.amount.cmp(&b.amount),
        SortField::Merchant => a.merchant.cmp(&b.merchant),
        SortField::Category => a.category.cmp(&b.category),
    };
    
    match order {
        SortOrder::Ascending => ordering,
        SortOrder::Descending => ordering.reverse(),
    }
}

impl App {
    pub fn toggle_sort_order(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
        self.sort_transactions();
    }

    pub fn handle_input(&mut self, c: char) {
        match self.input_mode {
            InputMode::Filtering | InputMode::Categorizing => {
                self.input_text.push(c);
            }
            InputMode::Normal => {}
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.input_mode {
            InputMode::Filtering | InputMode::Categorizing => {
                self.input_text.pop();
            }
            InputMode::Normal => {}
        }
    }

    pub fn submit_input(&mut self) {
        match self.input_mode {
            InputMode::Filtering => {
                if !self.input_text.is_empty() {
                    self.apply_filter(self.input_text.clone());
                } else {
                    self.clear_filter();
                }
            }
            InputMode::Categorizing => {
                if let Some(idx) = self.selected_transaction {
                    if let Some(transaction) = self.transactions.get_mut(idx) {
                        if let Some(cat_idx) = self.category_selection {
                            if let Some(category) = self.available_categories.get(cat_idx) {
                                transaction.category = Some(category.as_str().to_string());
                                self.update_category_totals();
                            }
                        }
                    }
                }
                self.category_selection = None;
            }
            InputMode::Normal => {}
        }
        self.input_text.clear();
        self.input_mode = InputMode::Normal;
    }

    pub fn apply_filter(&mut self, filter: String) {
        self.filter = Some(filter.to_lowercase());
        self.filtered_transactions = self.transactions
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                let filter = self.filter.as_ref().unwrap();
                t.merchant.to_lowercase().contains(filter) ||
                t.description.to_lowercase().contains(filter) ||
                t.category.as_ref().map(|c| c.to_lowercase().contains(filter)).unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();
        
        if !self.filtered_transactions.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn clear_filter(&mut self) {
        self.filter = None;
        self.filtered_transactions.clear();
        if !self.transactions.is_empty() {
            self.list_state.select(Some(0));
        }
    }

    pub fn handle_category_selection(&mut self, key: KeyCode) {
        match key {
            KeyCode::Up => {
                if let Some(current) = self.category_selection {
                    self.category_selection = Some(if current == 0 {
                        self.available_categories.len() - 1
                    } else {
                        current - 1
                    });
                }
            }
            KeyCode::Down => {
                if let Some(current) = self.category_selection {
                    self.category_selection = Some(if current >= self.available_categories.len() - 1 {
                        0
                    } else {
                        current + 1
                    });
                }
            }
            _ => {}
        }
    }

    pub fn categorize_all_transactions(&mut self) {
        for transaction in &mut self.transactions {
            let category = Category::categorize_transaction(
                &self.categories,
                &transaction.merchant,
                &transaction.description
            );
            transaction.category = category;
        }
    }

    pub fn update_category_totals(&mut self) {
        let mut totals = HashMap::new();
        
        for transaction in &self.transactions {
            let category = transaction.category.as_deref().unwrap_or("Uncategorized").to_string();
            *totals.entry(category).or_insert(Decimal::ZERO) += transaction.amount;
        }

        self.category_totals = totals;
    }
}