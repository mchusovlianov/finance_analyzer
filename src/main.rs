use std::io;
use anyhow::Result;
use chrono::NaiveDateTime;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame, Terminal,
};
use rust_decimal::Decimal;
use serde::Deserialize;

mod csv;

#[derive(Debug, Deserialize)]
struct Transaction {
    id: u64,
    date: NaiveDateTime,
    amount: Decimal,
    merchant: String,
    description: String,
    category: Option<String>,
}

#[derive(Debug)]
struct Category {
    name: String,
    rules: Vec<Rule>,
}

#[derive(Debug)]
struct Rule {
    pattern: String,
    category: String,
    priority: u8,
}

#[derive(Debug)]
enum View {
    TransactionList,
    CategorySummary,
    TransactionDetail,
    CategoryDetail,
}

#[derive(Debug, PartialEq, Clone)]
enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, PartialEq, Clone)]
enum SortField {
    Date,
    Amount,
    Merchant,
    Category,
}

#[derive(Debug, PartialEq)]
enum InputMode {
    Normal,
    Filtering,
    Categorizing,
}

#[derive(Debug)]
struct App {
    transactions: Vec<Transaction>,
    filtered_transactions: Vec<usize>,
    categories: std::collections::HashMap<String, Category>,
    rules: Vec<Rule>,
    current_view: View,
    selected_transaction: Option<usize>,
    category_totals: std::collections::HashMap<String, Decimal>,
    list_state: ratatui::widgets::ListState,
    sort_field: SortField,
    sort_order: SortOrder,
    input_mode: InputMode,
    input_text: String,
    filter: Option<String>,
    can_show_details: bool,
}

impl Category {
    fn new(name: &str, patterns: &[(&str, u8)]) -> Self {
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
}

impl App {
    fn new(csv_path: &str) -> Result<Self> {
        let transactions = csv::read_transactions_from_csv(csv_path)?;
        let mut list_state = ratatui::widgets::ListState::default();
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
            rules: Vec::new(),
            current_view: View::TransactionList,
            selected_transaction: None,
            category_totals: std::collections::HashMap::new(),
            list_state,
            sort_field: SortField::Date,
            sort_order: SortOrder::Descending,
            input_mode: InputMode::Normal,
            input_text: String::new(),
            filter: None,
            can_show_details: false,
        };

        // Apply initial categorization
        app.categorize_all_transactions();
        app.update_category_totals();

        Ok(app)
    }

    fn categorize_all_transactions(&mut self) {
        let mut all_rules: Vec<&Rule> = self.categories.values()
            .flat_map(|c| c.rules.iter())
            .collect();
        
        // Sort rules by priority (higher priority first)
        all_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        for transaction in &mut self.transactions {
            let merchant_lower = transaction.merchant.to_lowercase();
            let description_lower = transaction.description.to_lowercase();

            for rule in &all_rules {
                let pattern_lower = rule.pattern.to_lowercase();
                if merchant_lower.contains(&pattern_lower) || description_lower.contains(&pattern_lower) {
                    transaction.category = Some(rule.category.clone());
                    break;
                }
            }
        }
    }

    fn update_category_totals(&mut self) {
        let mut totals = std::collections::HashMap::new();
        
        for transaction in &self.transactions {
            let category = transaction.category.as_deref().unwrap_or("Uncategorized").to_string();
            *totals.entry(category).or_insert(Decimal::ZERO) += transaction.amount;
        }

        self.category_totals = totals;
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

    fn sort_transactions(&mut self) {
        let sort_field = self.sort_field.clone();
        let sort_order = self.sort_order.clone();
        
        if !self.filtered_transactions.is_empty() {
            self.filtered_transactions.sort_by(|&a, &b| {
                let ta = &self.transactions[a];
                let tb = &self.transactions[b];
                compare_transactions(ta, tb, &sort_field, &sort_order)
            });
        } else {
            self.transactions.sort_by(|a, b| {
                compare_transactions(a, b, &sort_field, &sort_order)
            });
        }
    }

    fn toggle_sort_order(&mut self) {
        self.sort_order = match self.sort_order {
            SortOrder::Ascending => SortOrder::Descending,
            SortOrder::Descending => SortOrder::Ascending,
        };
        self.sort_transactions();
    }

    fn handle_input(&mut self, c: char) {
        match self.input_mode {
            InputMode::Filtering | InputMode::Categorizing => {
                self.input_text.push(c);
            }
            InputMode::Normal => {}
        }
    }

    fn handle_backspace(&mut self) {
        match self.input_mode {
            InputMode::Filtering | InputMode::Categorizing => {
                self.input_text.pop();
            }
            InputMode::Normal => {}
        }
    }

    fn submit_input(&mut self) {
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
                        if !self.input_text.is_empty() {
                            transaction.category = Some(self.input_text.clone());
                        }
                    }
                }
            }
            InputMode::Normal => {}
        }
        self.input_text.clear();
        self.input_mode = InputMode::Normal;
    }

    fn apply_filter(&mut self, filter: String) {
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

    fn clear_filter(&mut self) {
        self.filter = None;
        self.filtered_transactions.clear();
        if !self.transactions.is_empty() {
            self.list_state.select(Some(0));
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

impl Transaction {
    fn to_list_item(&self) -> ListItem {
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

fn render_transaction_list(f: &mut Frame, app: &App, area: Rect) {
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

fn render_popup(f: &mut Frame, app: &App, area: Rect) {
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
    
    // Add semi-transparent background
    let shadow_area = popup_area;
    f.render_widget(Clear, shadow_area);
    
    // Clear the exact popup area
    f.render_widget(Clear, popup_area);
    
    // Render the popup with its content
    f.render_widget(paragraph, popup_area);
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

fn render_input_prompt(f: &mut Frame, app: &App, area: Rect) {
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
    
    // Add shadow effect
    let shadow_area = Rect::new(
        popup_area.x + 1,
        popup_area.y + 1,
        popup_area.width,
        popup_area.height
    );
    f.render_widget(Clear, shadow_area);
    f.render_widget(Block::default().style(Style::default().bg(Color::DarkGray)), shadow_area);
    
    f.render_widget(Clear, popup_area);
    f.render_widget(input, popup_area);
}

fn render_category_summary(f: &mut Frame, app: &App, area: Rect) {
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

    // Sort items by absolute amount
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

fn render_help_panel(f: &mut Frame, area: Rect) {
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

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            
            // Create layout with help panel at bottom
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                ].as_ref())
                .split(size);

            match app.current_view {
                View::TransactionList => render_transaction_list(f, &app, chunks[0]),
                View::CategorySummary => render_category_summary(f, &app, chunks[0]),
                View::TransactionDetail => render_transaction_list(f, &app, chunks[0]),
                View::CategoryDetail => render_category_summary(f, &app, chunks[0]),
            }
            
            render_help_panel(f, chunks[1]);

            // Render popups if needed
            if matches!(app.current_view, View::TransactionDetail) {
                render_popup(f, &app, size);
            }

            // Always render input prompt on top if in input mode
            if app.input_mode != InputMode::Normal {
                render_input_prompt(f, &app, size);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Tab => {
                            app.current_view = match app.current_view {
                                View::TransactionList => View::CategorySummary,
                                View::CategorySummary => View::TransactionList,
                                View::TransactionDetail => View::TransactionList,
                                View::CategoryDetail => View::CategorySummary,
                            };
                        }
                        KeyCode::Char('d') => {
                            if let View::TransactionList = app.current_view {
                                app.current_view = if matches!(app.current_view, View::TransactionDetail) {
                                    View::TransactionList
                                } else {
                                    View::TransactionDetail
                                };
                            }
                        },
                        KeyCode::Esc => {
                            if let View::TransactionDetail = app.current_view {
                                app.current_view = View::TransactionList;
                            }
                        }
                        KeyCode::Up => app.previous(),
                        KeyCode::Down => app.next(),
                        KeyCode::Char('s') => app.toggle_sort_order(),
                        KeyCode::Char('f') => {
                            app.input_mode = InputMode::Filtering;
                        }
                        KeyCode::Char('c') => {
                            if app.selected_transaction.is_some() {
                                app.input_mode = InputMode::Categorizing;
                            }
                        }
                        _ => {}
                    },
                    InputMode::Filtering | InputMode::Categorizing => match key.code {
                        KeyCode::Enter => app.submit_input(),
                        KeyCode::Esc => {
                            app.input_text.clear();
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Backspace => app.handle_backspace(),
                        KeyCode::Char(c) => app.handle_input(c),
                        _ => {}
                    },
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let csv_path = args.get(1).ok_or_else(|| {
        anyhow::anyhow!("Please provide a CSV file path as an argument\nUsage: finance-analyzer <csv-file-path>")
    })?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new(csv_path)?;
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}
