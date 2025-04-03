use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListItem,
};

#[derive(Debug, serde::Deserialize)]
pub struct Transaction {
    pub id: u64,
    pub date: NaiveDateTime,
    pub amount: Decimal,
    pub merchant: String,
    pub description: String,
    pub category: Option<String>,
}

impl Transaction {
    pub fn to_list_item(&self) -> ListItem {
        let amount_style = if self.amount < Decimal::ZERO {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };

        ListItem::new(Line::from(vec![
            Span::raw(format!("{:<10} ", self.date.format("%Y-%m-%d"))),
            Span::styled(format!("{:>10} ", self.amount), amount_style),
            Span::raw(format!("{:<30} ", self.merchant)),
            Span::raw(self.category.as_deref().unwrap_or("Uncategorized")),
        ]))
    }
}