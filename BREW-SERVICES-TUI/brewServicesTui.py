import subprocess
from rich.console import Console
from rich.table import Table
from rich.prompt import Prompt
from rich import box

console = Console()

def get_brew_services():
    result = subprocess.run(["brew", "services", "list"], capture_output=True, text=True)
    lines = result.stdout.strip().split("\n")[1:]  # Skip the header line
    services = []
    for line in lines:
        parts = line.split()
        service = {
            "name": parts[0],
            "status": parts[1],
            "user": parts[2] if len(parts) > 2 else "",
        }
        services.append(service)
    return services

def display_services(services):
    table = Table(title="Brew Services", box=box.ROUNDED)
    table.add_column("Service Name", style="cyan", no_wrap=True)
    table.add_column("Status", style="bold", no_wrap=True)
    table.add_column("User", style="magenta")

    for service in services:
        status_color = "green" if service["status"] == "started" else "red"
        table.add_row(service["name"], f"[{status_color}]{service['status']}[/{status_color}]", service["user"])

    console.print(table)

def manage_service(action, service_name):
    subprocess.run(["brew", "services", action, service_name])

def main():
    while True:
        services = get_brew_services()
        display_services(services)
        
        console.print("\nOptions: start [service], stop [service], restart [service], quit")
        user_input = Prompt.ask("Enter your command")
        if user_input == "quit":
            break
        
        parts = user_input.split()
        if len(parts) != 2 or parts[0] not in ["start", "stop", "restart"]:
            console.print("[red]Invalid command. Try again.[/red]")
            continue
        
        action, service_name = parts
        manage_service(action, service_name)

if __name__ == "__main__":
    main()

