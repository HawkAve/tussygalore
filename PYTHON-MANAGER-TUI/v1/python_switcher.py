import subprocess
from rich.console import Console
from rich.table import Table
from rich.markdown import Markdown

console = Console()

def get_python_versions():
    result = subprocess.run(['pyenv', 'versions', '--bare'], capture_output=True, text=True)
    versions = result.stdout.split()
    return versions

def get_pip_list(version):
    command = f'PYENV_VERSION={version} pyenv exec pip list'
    console.print(f"[bold]Executing:[/bold] {command} in {version}")
    result = subprocess.run(command, capture_output=True, text=True, shell=True)
    console.print(f"[bold]pip list output for {version}:[/bold] {result.stdout}")

    lines = result.stdout.splitlines()
    pip_list = []
    for line in lines[2:]:  # Skip the header lines
        parts = line.split()
        if len(parts) == 2:
            package, version = parts
            pip_list.append({'name': package, 'version': version})
    return pip_list

def get_package_location(version, package):
    command = f'PYENV_VERSION={version} pyenv exec python -c "import {package}; print({package}.__file__)"'
    console.print(f"[bold]Executing:[/bold] {command} in {version}")
    result = subprocess.run(command, capture_output=True, text=True, shell=True)
    location = result.stdout.strip()
    console.print(f"[bold]Location for package {package} in {version}:[/bold] {location}")
    return location

def create_table(version, pip_list):
    table = Table(title=f"Python {version} Packages")

    table.add_column("Index", justify="right", style="cyan", no_wrap=True)
    table.add_column("Package", style="magenta")
    table.add_column("Version", style="green")
    table.add_column("Location", style="blue", justify="left")

    for idx, package in enumerate(pip_list):
        location = get_package_location(version, package['name'])
        table.add_row(str(idx + 1), package['name'], package['version'], f"[link=file://{location}]{location}[/link]")

    return table

def main():
    versions = get_python_versions()
    for version in versions:
        pip_list = get_pip_list(version)
        if pip_list:
            table = create_table(version, pip_list)
            console.print(table)
        else:
            console.print(f"[bold red]No packages found for Python {version}[/bold red]")
        console.print(Markdown("\n---\n"))

if __name__ == "__main__":
    main()

