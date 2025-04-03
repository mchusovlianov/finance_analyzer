use std::io;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

use finance_analyzer::{
    ui::{
        app::{App, InputMode, View},
        render::{
            render_transaction_list, render_popup, render_category_summary,
            render_help_panel, render_category_selection, render_input_prompt,
        },
    },
};

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    mut app: App,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            
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

            if matches!(app.current_view, View::TransactionDetail) {
                render_popup(f, &app, size);
            }

            if app.input_mode != InputMode::Normal {
                render_input_prompt(f, &app, size);
            }

            if app.input_mode == InputMode::Categorizing {
                render_category_selection(f, &app, size);
            }
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('c') => {
                                if app.selected_transaction.is_some() {
                                    app.input_mode = InputMode::Categorizing;
                                    app.category_selection = Some(0);
                                }
                            }
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
                            }
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
                            _ => {}
                        }
                    }
                    InputMode::Categorizing => {
                        match key.code {
                            KeyCode::Enter => app.submit_input(),
                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                                app.category_selection = None;
                            }
                            KeyCode::Up | KeyCode::Down => app.handle_category_selection(key.code),
                            _ => {}
                        }
                    }
                    InputMode::Filtering => {
                        match key.code {
                            KeyCode::Enter => app.submit_input(),
                            KeyCode::Esc => {
                                app.input_text.clear();
                                app.input_mode = InputMode::Normal;
                            }
                            KeyCode::Backspace => app.handle_backspace(),
                            KeyCode::Char(c) => app.handle_input(c),
                            _ => {}
                        }
                    }
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

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(csv_path)?;
    let res = run_app(&mut terminal, app);

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
