use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::error::Error;
use std::io;
use std::process::Command;
use tui::backend::{Backend, CrosstermBackend};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Terminal;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
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
        println!("{:?}", err)
    }

    Ok(())
}

struct App {
    versions: Vec<String>,
    selected: usize,
}

impl App {
    fn new() -> App {
        let versions = list_python_versions();
        App {
            versions,
            selected: 0,
        }
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(size);

            let block = Block::default()
                .title("Python Version TUI")
                .borders(Borders::ALL);
            f.render_widget(block, chunks[0]);

            let items: Vec<ListItem> = app.versions.iter().map(|v| ListItem::new(v.as_str())).collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Versions"))
                .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[1], &mut app.selected);

            let paragraph = Paragraph::new("Press 'q' to quit")
                .style(Style::default().fg(Color::LightCyan))
                .block(Block::default().borders(Borders::ALL).title("Instructions"));
            f.render_widget(paragraph, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Down => {
                    if app.selected < app.versions.len() - 1 {
                        app.selected += 1;
                    }
                }
                KeyCode::Up => {
                    if app.selected > 0 {
                        app.selected -= 1;
                    }
                }
                _ => {}
            }
        }
    }
}

fn list_python_versions() -> Vec<String> {
    let output = Command::new("pyenv")
        .arg("versions")
        .output()
        .expect("Failed to execute pyenv command");

    let pyenv_versions = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .collect::<Vec<String>>();

    let output = Command::new("brew")
        .arg("list")
        .arg("--versions")
        .output()
        .expect("Failed to execute brew command");

    let brew_versions = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.starts_with("python@"))
        .map(|line| line.split_whitespace().nth(1).unwrap().to_string())
        .collect::<Vec<String>>();

    [pyenv_versions, brew_versions].concat()
}

