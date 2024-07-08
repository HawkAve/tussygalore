use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io, process::Command, time::{Duration, Instant}};

// Define a struct to hold the state of a scrollable list
struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

// Define the application state
struct App {
    installed_versions: StatefulList<String>,
    available_versions: StatefulList<String>,
    packages: Vec<String>,
    active_version: Option<String>,
    active_column: usize,
}

impl App {
    fn new() -> App {
        // Fetch installed Python versions using `pyenv versions`
        let installed_output = Command::new("pyenv")
            .arg("versions")
            .output()
            .expect("Failed to execute pyenv command");
        let installed_str = String::from_utf8_lossy(&installed_output.stdout);
        let installed_versions = installed_str
            .lines()
            .map(|line| line.replace("*", "").trim().split_whitespace().next().unwrap_or("").to_string())
            .filter(|line| !line.is_empty())
            .collect();

        // Fetch available Python versions using `pyenv install --list`
        let available_output = Command::new("pyenv")
            .arg("install")
            .arg("--list")
            .output()
            .expect("Failed to execute pyenv command");
        let available_str = String::from_utf8_lossy(&available_output.stdout);
        let mut available_versions: Vec<String> = available_str
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.contains("Anaconda")) // Filter out empty lines and Anaconda
            .collect();
        available_versions.reverse(); // Reverse to show latest first

        let active_version = get_active_python_version();

        let packages = if let Some(version) = &active_version {
            fetch_packages(version)
        } else {
            vec!["No version selected".to_string()]
        };

        App {
            installed_versions: StatefulList::with_items(installed_versions),
            available_versions: StatefulList::with_items(available_versions),
            packages,
            active_version,
            active_column: 0,
        }
    }

    fn next_item(&mut self) {
        if self.active_column == 0 {
            self.installed_versions.next();
            if let Some(selected) = self.installed_versions.state.selected() {
                let version = &self.installed_versions.items[selected];
                self.packages = fetch_packages(version);
            }
        } else if self.active_column == 1 {
            self.available_versions.next();
        }
    }

    fn previous_item(&mut self) {
        if self.active_column == 0 {
            self.installed_versions.previous();
            if let Some(selected) = self.installed_versions.state.selected() {
                let version = &self.installed_versions.items[selected];
                self.packages = fetch_packages(version);
            }
        } else if self.active_column == 1 {
            self.available_versions.previous();
        }
    }

    fn move_left(&mut self) {
        if self.active_column > 0 {
            self.active_column -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.active_column < 2 {
            self.active_column += 1;
        }
    }

    fn get_status_info(&self) -> String {
        let python_version = get_active_python_version().unwrap_or_else(|| "Unknown".to_string());
        let python_env = Command::new("which")
            .arg("python")
            .output()
            .expect("Failed to get Python environment")
            .stdout;
        let python_env = String::from_utf8_lossy(&python_env).trim().to_string();
        let venv_info = Command::new("pip")
            .arg("-V")
            .output()
            .expect("Failed to get virtual environment info")
            .stdout;
        let venv_info = String::from_utf8_lossy(&venv_info).trim().to_string();

        format!(
            "Current Python Version: {}\nPython Environment: {}\nVirtual Environment: {}",
            python_version, python_env, venv_info
        )
    }
}

fn get_active_python_version() -> Option<String> {
    let output = Command::new("pyenv")
        .arg("version-name")
        .output()
        .expect("Failed to get current Python version");
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn fetch_packages(version: &str) -> Vec<String> {
    // Use pyenv to activate the selected version and fetch its packages using pip
    let _ = Command::new("pyenv")
        .arg("shell")
        .arg(version)
        .output()
        .expect("Failed to activate Python version with pyenv");

    let output = Command::new("pip")
        .arg("list")
        .output()
        .expect("Failed to execute pip command");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let packages: Vec<String> = output_str
        .lines()
        .skip(2) // Skip the header lines
        .map(|line| line.trim().to_string())
        .collect();

    packages
}

// Main application loop
fn run_app<B: tui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        // Draw the UI
        terminal.draw(|f| {
            // Get the size of the terminal window
            let size = f.size();
            
            // Create a layout with three vertical columns and a bottom row for status
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Percentage(80),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(size);

            let column_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(34),
                    ]
                    .as_ref(),
                )
                .split(chunks[0]);

            // Create a list of items for each installed Python version
            let installed_items: Vec<ListItem> = app
                .installed_versions
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let style = if Some(i) == app.installed_versions.state.selected() {
                        Style::default().fg(Color::Magenta)
                    } else if app.active_version.as_deref() == Some(item) {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    ListItem::new(Spans::from(Span::styled(item, style)))
                })
                .collect();

            // Create a list of items for each available Python version
            let available_items: Vec<ListItem> = app
                .available_versions
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    let style = if Some(i) == app.available_versions.state.selected() {
                        Style::default().fg(Color::Magenta)
                    } else if app.active_version.as_deref() == Some(item) {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    ListItem::new(Spans::from(Span::styled(item, style)))
                })
                .collect();

            // Create a list of items for each package
            let package_items: Vec<ListItem> = app
                .packages
                .iter()
                .map(|i| ListItem::new(Spans::from(Span::styled(i, Style::default().fg(Color::Yellow)))))
                .collect();

            // Create a list widget with a title and borders for installed versions
            let installed_list = List::new(installed_items)
                .block(Block::default().borders(Borders::ALL).title("Python Versions"))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            // Create a list widget with a title and borders for available versions
            let available_list = List::new(available_items)
                .block(Block::default().borders(Borders::ALL).title("Available Versions"))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            // Create a list widget with a title and borders for packages
            let packages_list = List::new(package_items)
                .block(Block::default().borders(Borders::ALL).title("Packages"))
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            // Render the installed versions list in the first column
            if app.active_column == 0 {
                f.render_stateful_widget(installed_list, column_chunks[0], &mut app.installed_versions.state);
            } else {
                f.render_widget(installed_list, column_chunks[0]);
            }

            // Render the available versions list in the second column
            if app.active_column == 1 {
                f.render_stateful_widget(available_list, column_chunks[1], &mut app.available_versions.state);
            } else {
                f.render_widget(available_list, column_chunks[1]);
            }

            // Render the packages list in the third column
            f.render_widget(packages_list, column_chunks[2]);

            // Create and render the status box
            let status_text = app.get_status_info();
            let status_paragraph = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title("Status"))
                .style(Style::default().fg(Color::White).bg(Color::Black));
            f.render_widget(status_paragraph, chunks[1]);
        })?;

        // Handle keyboard events
        if crossterm::event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    // Exit the application if 'q' is pressed
                    KeyCode::Char('q') => {
                        disable_raw_mode()?;
                        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
                        return Ok(());
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous_item(),
                    KeyCode::Char('h') => app.move_left(),
                    KeyCode::Char('l') => app.move_right(),
                    _ => {}
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    
    // Create a new backend and terminal interface
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize the application and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal to its previous state
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Print any errors that occurred
    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

