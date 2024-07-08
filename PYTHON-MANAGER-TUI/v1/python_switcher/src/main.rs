use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span},
    widgets::{Block, Borders, Row, Table},
    Terminal,
};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize, Debug)]
struct Package {
    name: String,
    version: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let python_versions = get_python_versions();
    let mut packages = vec![];

    for version in &python_versions {
        if let Some(pip_list) = get_pip_list(version) {
            packages.push((version.clone(), pip_list));
        }
    }

    let app = App { packages };
    let res = run_app(&mut terminal, app);

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
    packages: Vec<(String, Vec<Package>)>,
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let mut rows = vec![];

            for (version, packages) in &app.packages {
                rows.push(Row::new(vec![
                    Span::styled(format!("Python {}", version), Style::default().fg(Color::Yellow)),
                    Span::styled("", Style::default()),
                    Span::styled("", Style::default()),
                ]));
                for (i, package) in packages.iter().enumerate() {
                    rows.push(Row::new(vec![
                        Span::styled(i.to_string(), Style::default().fg(Color::White)),
                        Span::styled(&package.name, Style::default().fg(Color::Green)),
                        Span::styled(&package.version, Style::default().fg(Color::Blue)),
                    ]));
                }
            }

            let table = Table::new(rows)
                .header(Row::new(vec![
                    Span::styled("Index", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("Package", Style::default().add_modifier(Modifier::BOLD)),
                    Span::styled("Version", Style::default().add_modifier(Modifier::BOLD)),
                ]))
                .block(Block::default().borders(Borders::ALL).title("Python Packages"))
                .widths(&[
                    Constraint::Length(10),
                    Constraint::Length(30),
                    Constraint::Length(30),
                ]);

            f.render_widget(table, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                return Ok(());
            }
        }
    }
}

fn get_python_versions() -> Vec<String> {
    let output = Command::new("pyenv")
        .arg("versions")
        .arg("--bare")
        .output()
        .expect("Failed to execute pyenv versions");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().map(|s| s.to_string()).collect()
}

fn get_pip_list(version: &str) -> Option<Vec<Package>> {
    let output = Command::new("pyenv")
        .arg("exec")
        .arg("pip")
        .arg("list")
        .arg("--format=json")
        .env("PYENV_VERSION", version)
        .output()
        .expect("Failed to execute pip list");
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let packages: Vec<Package> = serde_json::from_str(&stdout).ok()?;
        Some(packages)
    } else {
        None
    }
}

