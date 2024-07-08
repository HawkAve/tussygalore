use reqwest::Client;
use serde::Deserialize;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;
use termion::screen::AlternateScreen;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Deserialize)]
struct PyPISearchResult {
    name: String,
}

async fn search_pypi(query: &str) -> Vec<String> {
    log_debug(format!("Searching PyPI for: {}", query));
    let client = Client::new();
    let url = format!("https://pypi.org/search/?q={}", query);
    let response = match client.get(&url).send().await {
        Ok(res) => res,
        Err(e) => {
            log_debug(format!("Failed to fetch from PyPI: {}", e));
            return vec!["Failed to fetch packages from PyPI".to_string()];
        }
    };

    if response.status().is_success() {
        let text = match response.text().await {
            Ok(t) => t,
            Err(e) => {
                log_debug(format!("Failed to read response from PyPI: {}", e));
                return vec!["Failed to read response from PyPI".to_string()];
            }
        };
        let document = Html::parse_document(&text);
        let selector = match Selector::parse(".package-snippet") {
            Ok(sel) => sel,
            Err(e) => {
                log_debug(format!("Failed to parse HTML: {:?}", e));
                return vec!["Failed to parse HTML".to_string()];
            }
        };

        let packages: Vec<String> = document
            .select(&selector)
            .map(|element| element.value().attr("data-name").unwrap_or("").to_string())
            .collect();

        log_debug(format!("Found packages: {:?}", packages));
        packages
    } else {
        log_debug(format!("Failed to fetch packages from PyPI: status code {}", response.status()));
        vec!["Failed to fetch packages from PyPI".to_string()]
    }
}

async fn install_package(package: &str) -> Result<(), Box<dyn std::error::Error>> {
    log_debug(format!("Installing package: {}", package));
    let output = Command::new("pip")
        .arg("install")
        .arg(package)
        .output()?;
    
    if output.status.success() {
        log_debug(format!("Successfully installed package: {}", package));
        Ok(())
    } else {
        log_debug(format!("Failed to install package: {}", String::from_utf8_lossy(&output.stderr)));
        Err(format!("Failed to install package: {}", String::from_utf8_lossy(&output.stderr)).into())
    }
}

async fn get_python_versions() -> Vec<String> {
    let output = Command::new("pyenv")
        .arg("versions")
        .output()
        .expect("Failed to execute pyenv");
    let versions = String::from_utf8_lossy(&output.stdout);
    versions
        .lines()
        .map(|line| line.trim().replace("* ", "").to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

async fn get_packages_for_version(version: &str) -> Vec<String> {
    log_debug(format!("Fetching packages for version: {}", version));
    let clean_version = version.split_whitespace().next().unwrap_or("").to_string();
    let output = Command::new("pyenv")
        .arg("exec")
        .arg("pip")
        .arg("list")
        .arg("--format=columns")
        .env("PYENV_VERSION", &clean_version)
        .output()
        .expect("Failed to execute pip");

    if !output.status.success() {
        log_debug(format!("Failed to fetch packages for version {}: {}", version, String::from_utf8_lossy(&output.stderr)));
        return vec![format!("Failed to fetch packages: {}", String::from_utf8_lossy(&output.stderr))];
    }

    let packages = String::from_utf8_lossy(&output.stdout);
    log_debug(format!("Packages fetched for version {}: {}", version, packages));

    packages
        .lines()
        .skip(2) // Skip the header lines
        .map(|line| line.to_string())
        .collect()
}

fn log_debug(message: String) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .expect("Failed to open debug log file");
    writeln!(file, "{}", message).expect("Failed to write to debug log file");
}

async fn get_current_python_version() -> String {
    let output = Command::new("pyenv")
        .arg("global")
        .output()
        .expect("Failed to get current Python version");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

async fn get_python_env() -> String {
    let output = Command::new("which")
        .arg("python3")
        .output()
        .expect("Failed to get Python environment");
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.contains(".pyenv") {
        "Pyenv".to_string()
    } else if path.contains("/usr/local/") {
        "Homebrew".to_string()
    } else {
        "System".to_string()
    }
}

async fn get_virtual_env() -> String {
    let output = Command::new("pip")
        .arg("--version")
        .output()
        .expect("Failed to get pip version");
    let version_info = String::from_utf8_lossy(&output.stdout).trim().to_string();
    version_info.split_whitespace().last().unwrap_or("Unknown").to_string()
}

async fn draw_ui(
    terminal: &mut Terminal<TermionBackend<AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>>>,
    versions: Arc<Vec<String>>,
    selected_version_index: Arc<RwLock<usize>>,
    package_cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
    current_python_version: &str,
    python_env: &str,
    virtual_env: &str,
    show_popup: bool,
    popup_message: &str,
    pypi_packages: &Vec<String>,
    selected_package_index: usize,
    show_pypi: bool,
    loading: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let selected_version_index = *selected_version_index.read().await;
    let version_items: Vec<ListItem> = versions
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let style = if v.contains(&current_python_version) {
                Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)
            } else if i == selected_version_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(v.clone()).style(style)
        })
        .collect();

    let packages = package_cache.read().await.get(&versions[selected_version_index]).cloned().unwrap_or_else(|| vec![String::from("Loading...")]);
    let package_items: Vec<ListItem> = packages.iter().map(|p| ListItem::new(p.clone()).style(Style::default().fg(Color::Yellow))).collect();

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
            ].as_ref())
            .split(f.size());

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(chunks[0]);

        let versions_list = List::new(version_items)
            .block(Block::default().title("Python Versions").borders(Borders::ALL).style(Style::default().fg(Color::Cyan)));
        f.render_widget(versions_list, main_chunks[0]);

        let packages_list = List::new(package_items)
            .block(Block::default().title("Packages").borders(Borders::ALL).style(Style::default().fg(Color::Yellow)));
        f.render_widget(packages_list, main_chunks[1]);

        let status_text = format!(
            "Current Python Version: {}\nPython Environment: {}\nVirtual Environment: {}",
            current_python_version, python_env, virtual_env
        );
        let status_block = Paragraph::new(status_text)
            .block(Block::default().title("Status").borders(Borders::ALL));
        f.render_widget(status_block, chunks[2]);

        if show_popup {
            let popup_block = Paragraph::new(popup_message)
                .block(Block::default().title("Options").borders(Borders::ALL).style(Style::default().fg(Color::Red)));
            f.render_widget(popup_block, chunks[1]);
        }

        if show_pypi {
            let pypi_items: Vec<ListItem> = pypi_packages.iter().enumerate().map(|(i, p)| {
                let style = if i == selected_package_index {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(p.clone()).style(style)
            }).collect();

            let pypi_list = List::new(pypi_items)
                .block(Block::default().title("PyPI Packages").borders(Borders::ALL).style(Style::default().fg(Color::Green)));
            f.render_widget(pypi_list, chunks[1]);
        }

        if loading {
            let loading_block = Paragraph::new("Loading...").block(Block::default().title("Status").borders(Borders::ALL));
            f.render_widget(loading_block, chunks[2]);
        }
    })?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_debug("Starting application".to_string());
    let stdout = std::io::stdout().into_raw_mode()?;
    let stdin = termion::async_stdin();
    let backend = TermionBackend::new(AlternateScreen::from(stdout));
    let mut terminal = Terminal::new(backend)?;

    let versions = Arc::new(get_python_versions().await);
    let selected_version_index = Arc::new(RwLock::new(0));
    let package_cache: Arc<RwLock<HashMap<String, Vec<String>>>> = Arc::new(RwLock::new(HashMap::new()));
    let loading_packages: Arc<RwLock<bool>> = Arc::new(RwLock::new(true));

    let mut current_python_version = get_current_python_version().await;
    let python_env = get_python_env().await;
    let virtual_env = get_virtual_env().await;

    let mut show_popup = false;
    let mut popup_message = String::new();
    let mut show_pypi = false;
    let mut pypi_packages = Vec::new();
    let mut selected_package_index = 0;
    let mut loading = false;

    // Prefetch packages in the background
    {
        let package_cache_clone = Arc::clone(&package_cache);
        let versions_clone = Arc::clone(&versions);
        tokio::spawn(async move {
            for version in versions_clone.iter() {
                let pkgs = get_packages_for_version(version).await;
                package_cache_clone.write().await.insert(version.clone(), pkgs);
            }
        });
    }

    // Load initial packages
    {
        let package_cache_clone = Arc::clone(&package_cache);
        let loading_packages_clone = Arc::clone(&loading_packages);
        let version = versions[*selected_version_index.read().await].clone();
        tokio::spawn(async move {
            let pkgs = get_packages_for_version(&version).await;
            package_cache_clone.write().await.insert(version.clone(), pkgs);
            *loading_packages_clone.write().await = false;
        });
    }

    terminal.clear()?;
    let mut keys = stdin.keys();
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        let new_python_version = get_current_python_version().await;
        if new_python_version != current_python_version {
            current_python_version = new_python_version;
            let mut index = selected_version_index.write().await;
            if let Some(new_index) = versions.iter().position(|v| *v == current_python_version) {
                *index = new_index;
                let version = versions[*index].clone();
                let package_cache_clone = Arc::clone(&package_cache);
                tokio::spawn(async move {
                    let pkgs = get_packages_for_version(&version).await;
                    package_cache_clone.write().await.insert(version.clone(), pkgs);
                });
            }
        }

        draw_ui(
            &mut terminal,
            Arc::clone(&versions),
            Arc::clone(&selected_version_index),
            Arc::clone(&package_cache),
            &current_python_version,
            &python_env,
            &virtual_env,
            show_popup,
            &popup_message,
            &pypi_packages,
            selected_package_index,
            show_pypi,
            loading,
        ).await?;

        if let Some(Ok(key)) = keys.next() {
            match key {
                Key::Char('q') => {
                    terminal.clear()?;
                    log_debug("Exiting application".to_string());
                    return Ok(());
                }
                Key::Char('j') => {
                    let mut index = selected_version_index.write().await;
                    if show_pypi {
                        if selected_package_index < pypi_packages.len() - 1 {
                            selected_package_index += 1;
                        }
                    } else if *index < versions.len() - 1 {
                        *index += 1;
                        let version = versions[*index].clone();
                        let package_cache_clone = Arc::clone(&package_cache);
                        tokio::spawn(async move {
                            let pkgs = get_packages_for_version(&version).await;
                            package_cache_clone.write().await.insert(version.clone(), pkgs);
                        });
                    }
                }
                Key::Char('k') => {
                    let mut index = selected_version_index.write().await;
                    if show_pypi {
                        if selected_package_index > 0 {
                            selected_package_index -= 1;
                        }
                    } else if *index > 0 {
                        *index -= 1;
                        let version = versions[*index].clone();
                        let package_cache_clone = Arc::clone(&package_cache);
                        tokio::spawn(async move {
                            let pkgs = get_packages_for_version(&version).await;
                            package_cache_clone.write().await.insert(version.clone(), pkgs);
                        });
                    }
                }
                Key::Char('\n') => {
                    if show_pypi {
                        let selected_package = &pypi_packages[selected_package_index];
                        match install_package(selected_package).await {
                            Ok(_) => {
                                popup_message = format!("Successfully installed {}", selected_package);
                                log_debug(format!("Successfully installed {}", selected_package));
                            },
                            Err(e) => {
                                popup_message = format!("Failed to install {}: {}", selected_package, e);
                                log_debug(format!("Failed to install {}: {}", selected_package, e));
                            },
                        }
                        show_popup = true;
                        show_pypi = false;
                    } else {
                        show_popup = true;
                        popup_message = String::from("1. Switch to this version\n2. Add packages to this version\nPress 1 or 2 to choose, or q to cancel");
                    }
                }
                Key::Char('1') => {
                    if show_popup {
                        let index = *selected_version_index.read().await;
                        let version = versions[index].clone();
                        Command::new("pyenv")
                            .arg("global")
                            .arg(&version)
                            .output()
                            .expect("Failed to switch Python version");
                        current_python_version = get_current_python_version().await;
                        show_popup = false;
                    }
                }
                Key::Char('2') => {
                    if show_popup {
                        show_popup = false;
                        show_pypi = true;
                        loading = true;
                        pypi_packages = search_pypi("").await;
                        loading = false;
                    }
                }
                Key::Char('s') => {
                    if show_pypi {
                        loading = true;
                        pypi_packages = search_pypi("").await;
                        loading = false;
                    }
                }
                Key::Char('a') => {
                    if show_pypi {
                        let mut query = String::new();
                        while let Some(Ok(key)) = keys.next() {
                            match key {
                                Key::Char('\n') => break,
                                Key::Char(c) => query.push(c),
                                Key::Backspace => {
                                    query.pop();
                                }
                                _ => {}
                            }
                            loading = true;
                            pypi_packages = search_pypi(&query).await;
                            loading = false;
                        }
                    }
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

