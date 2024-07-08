from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.layout import Layout
from rich.progress import Progress, SpinnerColumn, BarColumn, TextColumn
from rich.text import Text
import subprocess
import os
import readchar
from concurrent.futures import ThreadPoolExecutor

# Ensure pyenv is in the PATH
pyenv_root = subprocess.run(["pyenv", "root"], capture_output=True, text=True).stdout.strip()
os.environ["PATH"] = f"{pyenv_root}/shims:{pyenv_root}/bin:" + os.environ["PATH"]

def run_command(command, env=None):
    result = subprocess.run(command, capture_output=True, text=True, env=env)
    return result.stdout.strip().splitlines(), result.stderr.strip().splitlines(), result.returncode

def get_pyenv_versions():
    return sorted(run_command(["pyenv", "versions", "--bare"])[0])

def get_available_pyenv_versions():
    versions, _, _ = run_command(["pyenv", "install", "--list"])
    return [v.strip() for v in versions if v.strip()]

def get_homebrew_versions():
    output = run_command(["brew", "list", "--versions", "python"])[0]
    versions = []
    for line in output:
        parts = line.split()
        if len(parts) > 1:
            versions.extend(parts[1:])
    return sorted(versions)

def get_current_pyenv_version():
    return run_command(["pyenv", "version-name"])[0][0]

def get_current_homebrew_version():
    return run_command(["python3", "--version"])[0][0].split()[1]

def get_pyenv_python_path(version):
    path = run_command(["pyenv", "which", "python"], env={**os.environ, "PYENV_VERSION": version})[0][0]
    return path if os.path.isfile(path) else None

def get_site_packages_directory(python_executable):
    try:
        output = run_command([python_executable, "-m", "site", "--user-site"])[0][0]
        return os.path.relpath(output, os.path.expanduser("~"))
    except Exception:
        return "Unknown"

def get_installed_packages(python_executable, max_packages=50):
    if not python_executable or not os.path.isfile(python_executable):
        return [("No executable found", "N/A")]
    result = run_command([python_executable, "-m", "pip", "list", "--format=freeze"])[0]
    packages = []
    for line in result[:max_packages]:  # Limit the number of displayed packages
        if "==" in line:
            package, version = line.split("==")
            packages.append((f"{package}=={version}", "N/A"))
    return packages

def create_version_panel(source, version, current_version, python_path, max_packages, current_package_page, items_per_page):
    display_version = f"{version} [green]✔[/green]" if version == current_version else version
    packages = get_installed_packages(python_path, max_packages)
    site_packages = get_site_packages_directory(python_path)
    
    table = Table(show_header=True, header_style="bold", box=None, padding=(0, 1))
    table.add_column("Source", style="bold", width=6)
    table.add_column("Version", style="cyan", width=10)
    table.add_column("Packages", style="yellow", width=40)
    table.add_column("Location", style="magenta", width=30)

    start_index = current_package_page * items_per_page
    end_index = start_index + items_per_page
    display_packages = packages[start_index:end_index]

    for i, (package, location) in enumerate(display_packages):
        package_name = package.split('==')[0]
        package_link = f"[link=yy {site_packages}/{package_name}]{package}[/link]"
        current_indicator = " [bold yellow]→[/bold yellow]" if i == current_package_page else ""
        table.add_row(source, display_version, f"{package_link}{current_indicator}", site_packages if location == "N/A" else location)
    
    return Panel(table, title=f"{source} Python Version", padding=(0, 0))

def load_panels(pyenv_versions, homebrew_versions, current_pyenv_version, current_homebrew_version, max_packages, current_package_page, items_per_page):
    panels = []
    version_list = []

    with ThreadPoolExecutor() as executor:
        future_to_version = {}
        futures = []
        
        for version in pyenv_versions:
            python_path = get_pyenv_python_path(version)
            future = executor.submit(create_version_panel, "pyenv", version, current_pyenv_version, python_path, max_packages, current_package_page, items_per_page)
            future_to_version[future] = version
            futures.append(future)
            version_list.append(version)
        
        for version in homebrew_versions:
            python_path = f"/usr/local/opt/python@{version}/bin/python3" if version != current_homebrew_version else "/usr/local/bin/python3"
            future = executor.submit(create_version_panel, "Homebrew", version, current_homebrew_version, python_path, max_packages, current_package_page, items_per_page)
            future_to_version[future] = version
            futures.append(future)
            version_list.append(version)

        for future in futures:
            panels.append(future.result())

    return panels, version_list

def create_version_list_table(version_list, current_version, current_page):
    table = Table(show_header=True, header_style="bold", box=None, padding=(0, 1))
    table.add_column("Python Version", style="cyan", width=20)

    for i, version in enumerate(version_list, 1):
        display_version = f"{version} [green]✔[/green]" if version == current_version else version
        current_indicator = " [bold yellow]→[/bold yellow]" if i - 1 == current_page else ""
        table.add_row(f"{display_version}{current_indicator}")

    return table

def create_available_versions_table(available_versions, current_page, items_per_page):
    table = Table(show_header=True, header_style="bold", box=None, padding=(0, 1))
    table.add_column("Available Versions", style="cyan", width=20)

    start_index = (current_page // items_per_page) * items_per_page
    end_index = start_index + items_per_page
    display_versions = available_versions[start_index:end_index]

    for i, version in enumerate(display_versions, start=start_index):
        current_indicator = " [bold yellow]→[/bold yellow]" if i == current_page else ""
        table.add_row(f"{version}{current_indicator}")

    return table

def switch_python_version(version, is_pyenv):
    if is_pyenv:
        run_command(["pyenv", "global", version])
    else:
        run_command(["brew", "unlink", "python"])
        run_command(["brew", "link", f"python@{version}"])

def install_pyenv_version(version):
    env = os.environ.copy()
    openssl_dir = subprocess.run(["brew", "--prefix", "openssl"], capture_output=True, text=True).stdout.strip()
    env["LDFLAGS"] = f"-L{openssl_dir}/lib"
    env["CPPFLAGS"] = f"-I{openssl_dir}/include"
    env["PKG_CONFIG_PATH"] = f"{openssl_dir}/lib/pkgconfig"

    with Progress(SpinnerColumn(), BarColumn(), TextColumn("[progress.description]{task.description}"), transient=True) as progress:
        task = progress.add_task("Installing...", total=None)
        stdout, stderr, returncode = run_command(["pyenv", "install", version], env=env)
        progress.update(task, completed=100)
    return stdout, stderr, returncode

def install_version_window(version):
    console = Console()
    console.clear()
    console.print(f"Installing Python {version}...\n", style="bold cyan")
    stdout, stderr, returncode = install_pyenv_version(version)
    if returncode == 0:
        console.print(f"Installation of Python {version} completed.\n", style="bold green")
    else:
        console.print(f"Installation of Python {version} failed.\n", style="bold red")
        console.print("\n".join(stderr), style="bold red")
    
    console.print("Press [bold cyan]u[/bold cyan] to use this version, [bold cyan]g[/bold cyan] to make it global, or [bold red]q[/bold red] to quit.")
    while True:
        key = readchar.readkey()
        if key == 'u':
            run_command(["pyenv", "shell", version])
            return 'use'
        elif key == 'g':
            run_command(["pyenv", "global", version])
            return 'global'
        elif key == 'q':
            return None

def check_openssl():
    openssl_check, _, _ = run_command(["brew", "list", "openssl"])
    if not openssl_check:
        console = Console()
        console.print("OpenSSL is not installed. Installing OpenSSL...\n", style="bold cyan")
        run_command(["brew", "install", "openssl"])

def main(width=80, height=50, max_packages=50, items_per_page=15):
    console = Console()

    check_openssl()

    def load_versions_and_panels(current_package_page):
        pyenv_versions = get_pyenv_versions()
        homebrew_versions = get_homebrew_versions()
        available_pyenv_versions = get_available_pyenv_versions()
        current_pyenv_version = get_current_pyenv_version()
        current_homebrew_version = get_current_homebrew_version()
        pages, version_list = load_panels(pyenv_versions, homebrew_versions, current_pyenv_version, current_homebrew_version, max_packages, current_package_page, items_per_page)
        return pyenv_versions, homebrew_versions, available_pyenv_versions, current_pyenv_version, current_homebrew_version, pages, version_list

    current_package_page = 0
    pyenv_versions, homebrew_versions, available_pyenv_versions, current_pyenv_version, current_homebrew_version, pages, version_list = load_versions_and_panels(current_package_page)

    # Determine the initial page based on the current Python version
    current_version = current_pyenv_version if current_pyenv_version in version_list else current_homebrew_version
    current_page = version_list.index(current_version) if current_version in version_list else 0
    available_page = 0
    focus_section = 'installed'  # 'installed', 'available', or 'packages'

    title_text = Text("""
    ██████╗  ██╗   ██╗          ███╗   ███╗ ███╗   ██╗  ██████╗  ██████╗ 
    ██╔══██╗ ╚██╗ ██╔╝          ████╗ ████║ ████╗  ██║ ██╔════╝  ██╔══██╗
    ██████╔╝  ╚████╔╝           ██╔████╔██║ ██╔██╗ ██║ ██║  ███╗ ██████╔╝
    ██╔═══╝    ╚██╔╝            ██║╚██╔╝██║ ██║╚██╗██║ ██║   ██║ ██╔══██╗
    ██║         ██║    ███████╗ ██║ ╚═╝ ██║ ██║ ╚████║ ╚██████╔╝ ██║  ██║
    ╚═╝         ╚═╝    ╚══════╝ ╚═╝     ╚═╝ ╚═╝  ╚═══╝  ╚═════╝  ╚═╝  ╚═╝
    """, style="bold magenta")
    title_text.stylize("bold magenta")
    title_text.wrap(console, width // 1)

    while True:
        version_list_table = create_version_list_table(version_list, current_version, current_page)
        available_versions_table = create_available_versions_table(available_pyenv_versions, available_page, items_per_page)
        layout = Layout()
        layout.split_row(
            Layout(Panel(version_list_table, title="Python Versions", padding=(0, 1)), size=width // 4),
            Layout(Panel(available_versions_table, title="Available Python Versions", padding=(0, 1)), size=width // 4),
            Layout(pages[current_page], size=width // 1)
        )
        console.clear()
        console.print(title_text, justify="left")
        console.print(layout, height=height)  # Set the height of the TUI display
        console.print("\nPress [bold cyan]Up/Down Arrow[/bold cyan] or [bold cyan]j/k[/bold cyan] to navigate, [bold cyan]h/l[/bold cyan] to switch columns, [bold cyan]p[/bold cyan] to focus on packages, [bold cyan]Enter[/bold cyan] to switch to selected version, [bold cyan]i[/bold cyan] to install selected version, [bold red]q[/bold red] to quit.")
        key = readchar.readkey()
        if key == 'q':
            break
        elif key in [readchar.key.UP, 'k']:  # Up arrow or 'k'
            if focus_section == 'installed':
                current_page = (current_page - 1) % len(pages)
            elif focus_section == 'available':
                available_page = (available_page - 1) % len(available_pyenv_versions)
            elif focus_section == 'packages':
                current_package_page = max(0, current_package_page - 1)
        elif key in [readchar.key.DOWN, 'j']:  # Down arrow or 'j'
            if focus_section == 'installed':
                current_page = (current_page + 1) % len(pages)
            elif focus_section == 'available':
                available_page = (available_page + 1) % len(available_pyenv_versions)
            elif focus_section == 'packages':
                current_package_page += 1
        elif key in ['h']:  # Left arrow or 'h'
            if focus_section == 'available':
                focus_section = 'installed'
            elif focus_section == 'packages':
                focus_section = 'available'
        elif key in ['l']:  # Right arrow or 'l'
            if focus_section == 'installed':
                focus_section = 'available'
            elif focus_section == 'available':
                focus_section = 'packages'
        elif key in ['p']:  # 'p' key to focus on packages
            focus_section = 'packages'
        elif key == readchar.key.ENTER:  # Enter key
            if focus_section == 'installed':
                selected_version = version_list[current_page]
                is_pyenv = selected_version in pyenv_versions
                switch_python_version(selected_version, is_pyenv)
                pyenv_versions, homebrew_versions, available_pyenv_versions, current_pyenv_version, current_homebrew_version, pages, version_list = load_versions_and_panels(current_package_page)  # Reload versions and panels after switching
                current_version = current_pyenv_version if current_pyenv_version in version_list else current_homebrew_version
                current_page = version_list.index(current_version) if current_version in version_list else 0
        elif key == 'i':  # 'i' key for installing a new version
            if focus_section == 'available':
                selected_version = available_pyenv_versions[available_page]
                result = install_version_window(selected_version)
                pyenv_versions, homebrew_versions, available_pyenv_versions, current_pyenv_version, current_homebrew_version, pages, version_list = load_versions_and_panels(current_package_page)  # Reload versions and panels after installation
                if result == 'use' or result == 'global':
                    current_version = current_pyenv_version if current_pyenv_version in version_list else current_homebrew_version
                    current_page = version_list.index(current_version) if current_version in version_list else 0

if __name__ == "__main__":
    main(width=120, height=50, max_packages=50, items_per_page=15)  # Adjust these values as needed

