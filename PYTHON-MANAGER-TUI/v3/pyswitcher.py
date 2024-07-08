import requests
from bs4 import BeautifulSoup
import sqlite3
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.layout import Layout
from rich.prompt import Prompt
from rich.live import Live
from rich.align import Align
from rich import box
import subprocess
import os

# Database setup
def setup_database():
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    
    cursor.execute('''CREATE TABLE IF NOT EXISTS packages
                      (name TEXT PRIMARY KEY, description TEXT)''')
    
    cursor.execute('''CREATE TABLE IF NOT EXISTS installed_packages
                      (version TEXT, name TEXT, version_number TEXT, location TEXT, description TEXT)''')
    
    cursor.execute('''CREATE TABLE IF NOT EXISTS python_versions
                      (version TEXT PRIMARY KEY, path TEXT)''')
    
    conn.commit()
    conn.close()

setup_database()

# Scrape PyPI for package information
def scrape_pypi():
    url = "https://pypi.org/simple/"
    response = requests.get(url)
    soup = BeautifulSoup(response.text, 'html.parser')
    
    packages = []
    for link in soup.find_all('a'):
        packages.append(link.text)
    
    # Store packages in the database
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    for package in packages:
        cursor.execute("INSERT OR IGNORE INTO packages (name) VALUES (?)", (package,))
    
    conn.commit()
    conn.close()

scrape_pypi()

# Get installed Python versions
def get_python_versions():
    versions = []
    result = subprocess.run(['pyenv', 'versions', '--bare'], capture_output=True, text=True)
    for version in result.stdout.splitlines():
        versions.append(version.strip())
    
    # Store versions in the database
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    for version in versions:
        cursor.execute("INSERT OR IGNORE INTO python_versions (version) VALUES (?)", (version,))
    
    conn.commit()
    conn.close()

get_python_versions()

# TUI setup with rich
console = Console()

def display_python_versions():
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    
    cursor.execute("SELECT version FROM python_versions")
    versions = cursor.fetchall()
    
    table = Table(title="Python Versions Installed", box=box.DOUBLE)
    table.add_column("Index", justify="center", style="cyan", no_wrap=True)
    table.add_column("Version", justify="center", style="cyan", no_wrap=True)
    
    for i, version in enumerate(versions, 1):
        table.add_row(str(i), version[0])
    
    conn.close()
    return table, versions

def display_installed_packages(version):
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    
    cursor.execute("SELECT name, version_number, location, description FROM installed_packages WHERE version=?", (version,))
    packages = cursor.fetchall()
    
    table = Table(title=f"Installed Packages for Python {version}", box=box.SQUARE)
    table.add_column("Package", justify="center", style="magenta", no_wrap=True)
    table.add_column("Version", justify="center", style="green")
    table.add_column("Location", justify="center", style="yellow")
    table.add_column("Description", justify="center", style="blue")
    
    for package in packages:
        table.add_row(package[0], package[1], package[2], package[3])
    
    conn.close()
    return table

def display_available_packages():
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    
    cursor.execute("SELECT name FROM packages")
    packages = cursor.fetchall()
    
    table = Table(title="Available Packages on PyPI", box=box.ROUNDED)
    table.add_column("Package", justify="center", style="cyan", no_wrap=True)
    
    for package in packages:
        table.add_row(package[0])
    
    conn.close()
    return table

def install_package(version, package_name):
    console.print(f"Installing {package_name} for Python {version}...")
    result = subprocess.run([f"pyenv exec pip install {package_name}"], shell=True, capture_output=True, text=True)
    if result.returncode == 0:
        console.print(f"[green]Successfully installed {package_name}[/green]")
        location_result = subprocess.run([f"pyenv exec pip show {package_name}"], shell=True, capture_output=True, text=True)
        location = None
        description = None
        for line in location_result.stdout.splitlines():
            if line.startswith("Location:"):
                location = line.split(":")[1].strip()
            if line.startswith("Summary:"):
                description = line.split(":")[1].strip()
        
        conn = sqlite3.connect('packages.db')
        cursor = conn.cursor()
        cursor.execute("INSERT INTO installed_packages (version, name, version_number, location, description) VALUES (?, ?, ?, ?, ?)",
                       (version, package_name, "latest", location, description))
        conn.commit()
        conn.close()
    else:
        console.print(f"[red]Failed to install {package_name}[/red]")

def browse_package_location(version):
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    cursor.execute("SELECT location FROM installed_packages WHERE version=?", (version,))
    locations = cursor.fetchall()
    conn.close()

    for location in locations:
        if location[0]:
            console.print(f"Browsing location: {location[0]}")
            subprocess.run(['yazi', location[0]])

def tui():
    layout = Layout()
    
    layout.split_column(
        Layout(name="header", size=3),
        Layout(name="body", ratio=1),
        Layout(name="footer", size=3)
    )
    
    layout["header"].update(Panel("Python Environment Manager", style="bold white on blue"))
    
    while True:
        table, versions = display_python_versions()
        layout["body"].update(table)
        console.print(layout)
        
        choice = Prompt.ask("Enter the number of the Python version to view details, 'q' to quit, or 'r' to refresh the list")
        if choice == 'q':
            break
        elif choice == 'r':
            continue
        
        try:
            index = int(choice) - 1
            if index < 0 or index >= len(versions):
                console.print("[red]Invalid choice, please try again.[/red]")
                continue
            version = versions[index][0]
        except ValueError:
            console.print("[red]Invalid input, please enter a number.[/red]")
            continue

        while True:
            layout["body"].update(display_installed_packages(version))
            console.print(layout)
            layout["body"].update(display_available_packages())
            console.print(layout)
            
            action = Prompt.ask("1: Install a package, 2: Browse package location, 'b' to go back")
            
            if action == 'b':
                break
            elif action == '1':
                package_name = Prompt.ask("Enter the package name to install")
                install_package(version, package_name)
            elif action == '2':
                browse_package_location(version)
            else:
                console.print("[red]Invalid choice, please try again.[/red]")

tui()

