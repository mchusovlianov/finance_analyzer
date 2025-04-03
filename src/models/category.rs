use std::collections::HashMap;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct Category {
    pub name: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug)]
pub struct Rule {
    pub pattern: String,
    pub category: String,
    pub priority: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CategoryType {
    Groceries,
    Utilities,
    Transportation,
    Childcare,
    Entertainment,
    Government,
    InternalTransfer,
    Shopping,
    Dining,
    Healthcare,
    Education,
    Travel,
    Uncategorized,
}

impl CategoryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CategoryType::Groceries => "Groceries",
            CategoryType::Utilities => "Utilities",
            CategoryType::Transportation => "Transportation",
            CategoryType::Childcare => "Childcare",
            CategoryType::Entertainment => "Entertainment",
            CategoryType::Government => "Government",
            CategoryType::InternalTransfer => "Internal Transfer",
            CategoryType::Shopping => "Shopping",
            CategoryType::Dining => "Dining",
            CategoryType::Healthcare => "Healthcare",
            CategoryType::Education => "Education",
            CategoryType::Travel => "Travel",
            CategoryType::Uncategorized => "Uncategorized",
        }
    }

    pub fn all() -> Vec<CategoryType> {
        vec![
            CategoryType::Groceries,
            CategoryType::Utilities,
            CategoryType::Transportation,
            CategoryType::Childcare,
            CategoryType::Entertainment,
            CategoryType::Government,
            CategoryType::InternalTransfer,
            CategoryType::Shopping,
            CategoryType::Dining,
            CategoryType::Healthcare,
            CategoryType::Education,
            CategoryType::Travel,
            CategoryType::Uncategorized,
        ]
    }
}

impl Category {
    pub fn new(name: &str, patterns: &[(&str, u8)]) -> Self {
        Category {
            name: name.to_string(),
            rules: patterns.iter()
                .map(|(pattern, priority)| Rule {
                    pattern: pattern.to_string(),
                    category: name.to_string(),
                    priority: *priority,
                })
                .collect(),
        }
    }

    pub fn categorize_transaction(categories: &HashMap<String, Category>, merchant: &str, description: &str) -> Option<String> {
        let mut all_rules: Vec<&Rule> = categories.values()
            .flat_map(|c| c.rules.iter())
            .collect();
        
        all_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        let merchant_lower = merchant.to_lowercase();
        let description_lower = description.to_lowercase();

        for rule in all_rules {
            let pattern_lower = rule.pattern.to_lowercase();
            if merchant_lower.contains(&pattern_lower) || description_lower.contains(&pattern_lower) {
                return Some(rule.category.clone());
            }
        }
        None
    }
}