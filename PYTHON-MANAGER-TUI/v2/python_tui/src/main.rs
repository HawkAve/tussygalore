
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem},
    layout::{Layout, Constraint, Direction},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create a list of Python versions (dummy data for now)
    let python_versions = vec![
        "Python 3.9.1 (pyenv)",
        "Python 3.8.5 (homebrew)",
        "Python 3.7.9 (pyenv)",
    ];

    // Main loop
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(size);

            let items: Vec<ListItem> = python_versions
                .iter()
                .map(|i| ListItem::new(*i))
                .collect();
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Python Versions"));
            f.render_widget(list, chunks[0]);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

