use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use rust_decimal::Decimal;

use super::app::{App, InputMode};

pub fn render_transaction_list(f: &mut Frame, app: &App, area: Rect) {
    let transactions = if app.filtered_transactions.is_empty() {
        &app.transactions
    } else {
        &app.transactions
    };

    let items: Vec<ListItem> = if app.filtered_transactions.is_empty() {
        transactions.iter().map(|t| t.to_list_item()).collect()
    } else {
        app.filtered_transactions.iter()
            .map(|&idx| transactions[idx].to_list_item())
            .collect()
    };

    let total_amount: Decimal = transactions.iter()
        .map(|t| t.amount)
        .sum();

    let header = format!(
        "Transactions ({} total, {} shown) Total: {:.2}", 
        app.transactions.len(),
        if app.filtered_transactions.is_empty() { app.transactions.len() } else { app.filtered_transactions.len() },
        total_amount
    );

    let list = List::new(items)
        .block(Block::default()
            .title(header)
            .borders(Borders::ALL))
        .highlight_style(Style::default()
            .add_modifier(Modifier::REVERSED)
            .add_modifier(Modifier::BOLD))
        .highlight_symbol("➤ ");

    f.render_stateful_widget(list, area, &mut app.list_state.clone());
}

pub fn render_popup(f: &mut Frame, app: &App, area: Rect) {
    let text = if let Some(idx) = app.selected_transaction {
        if let Some(transaction) = app.transactions.get(idx) {
            let amount_style = if transaction.amount < Decimal::ZERO {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            vec![
                Line::from(vec![
                    Span::raw("Date:       "),
                    Span::styled(
                        transaction.date.format("%Y-%m-%d").to_string(),
                        Style::default().add_modifier(Modifier::BOLD)
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Amount:     "),
                    Span::styled(
                        format!("{:.2}", transaction.amount),
                        amount_style.add_modifier(Modifier::BOLD)
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Merchant:   "),
                    Span::styled(
                        &transaction.merchant,
                        Style::default().add_modifier(Modifier::BOLD)
                    ),
                ]),
                Line::from(""),
                Line::from("Description:"),
                Line::from(transaction.description.clone()),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Category:   "),
                    Span::styled(
                        transaction.category.as_deref().unwrap_or("Uncategorized"),
                        Style::default().add_modifier(Modifier::BOLD)
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Esc", Style::default().fg(Color::Yellow)),
                    Span::raw(" close • "),
                    Span::styled("c", Style::default().fg(Color::Yellow)),
                    Span::raw(" change category"),
                ]),
            ]
        } else {
            vec![Line::from("No transaction selected")]
        }
    } else {
        vec![Line::from("No transaction selected")]
    };

    let block = Block::default()
        .title("Transaction Detail")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::White));

    let popup_area = centered_rect(60, 50, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

pub fn render_category_summary(f: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<(ListItem, Decimal)> = app.category_totals
        .iter()
        .map(|(category, total)| {
            let amount_style = if *total < Decimal::ZERO {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::Green)
            };

            (ListItem::new(Line::from(vec![
                Span::raw(format!("{:<30} ", category)),
                Span::styled(format!("{:>10.2}", total), amount_style),
            ])), *total)
        })
        .collect();

    items.sort_by(|a, b| b.1.abs().cmp(&a.1.abs()));

    let total_amount: Decimal = app.category_totals.values().sum();

    let list = List::new(items.into_iter().map(|(item, _)| item).collect::<Vec<_>>())
        .block(Block::default()
            .title(format!("Category Summary (Total: {:.2})", total_amount))
            .borders(Borders::ALL))
        .highlight_style(Style::default()
            .add_modifier(Modifier::REVERSED));

    f.render_widget(list, area);
}

pub fn render_help_panel(f: &mut Frame, area: Rect) {
    let text = vec![
        Line::from(vec![
            Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
            Span::raw(" Move • "),
            Span::styled("d", Style::default().fg(Color::Yellow)),
            Span::raw(" Details • "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(" Back • "),
            Span::styled("Tab", Style::default().fg(Color::Yellow)),
            Span::raw(" View • "),
            Span::styled("f", Style::default().fg(Color::Yellow)),
            Span::raw(" Filter • "),
            Span::styled("c", Style::default().fg(Color::Yellow)),
            Span::raw(" Category • "),
            Span::styled("s", Style::default().fg(Color::Yellow)),
            Span::raw(" Sort • "),
            Span::styled("q", Style::default().fg(Color::Yellow)),
            Span::raw(" Quit"),
        ]),
    ];

    let help = Paragraph::new(text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Help "))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(help, area);
}

pub fn render_category_selection(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app.available_categories
        .iter()
        .enumerate()
        .map(|(i, category)| {
            let style = if Some(i) == app.category_selection {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(category.as_str(), style)
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default()
            .title("Select Category (↑↓ to move, Enter to confirm, Esc to cancel)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let popup_area = centered_rect(40, 60, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(list, popup_area);
}

pub fn render_input_prompt(f: &mut Frame, app: &App, area: Rect) {
    if app.input_mode == InputMode::Normal {
        return;
    }

    let (title, placeholder) = match app.input_mode {
        InputMode::Filtering => ("Filter (Enter to apply, Esc to cancel)", "Enter text to filter transactions..."),
        InputMode::Categorizing => ("Categorize (Enter to apply, Esc to cancel)", "Enter category name..."),
        InputMode::Normal => return,
    };

    let input = Paragraph::new(if app.input_text.is_empty() {
        Line::from(placeholder).style(Style::default().fg(Color::DarkGray))
    } else {
        Line::from(app.input_text.as_str())
    })
    .block(Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow)));

    let popup_area = centered_rect(60, 10, area);
    f.render_widget(Clear, popup_area);
    f.render_widget(input, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(layout[1])[1]
}