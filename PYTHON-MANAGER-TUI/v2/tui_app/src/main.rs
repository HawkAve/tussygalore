use std::io;
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> Result<(), io::Error> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create a list of numbers from 1 to 10
    let items: Vec<ListItem> = (1..=10)
        .map(|i| ListItem::new(i.to_string()))
        .collect();

    // Create a list widget
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Numbers"));

    // Render the list
    terminal.draw(|f| {
        let size = f.size();
        f.render_widget(list, size);
    })?;

    // Wait for user input to exit
    loop {
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
