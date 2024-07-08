import os
from rich.console import Console
from rich.table import Table
from rich.prompt import Prompt

# Path to the zshrc file
zshrc_path = os.path.expanduser("~/.zshrc")
aliases = []

# Function to load aliases from the zshrc file
def load_aliases():
    global aliases
    aliases = []
    with open(zshrc_path, 'r') as file:
        lines = file.readlines()
    start, end = None, None
    for i, line in enumerate(lines):
        if line.strip() == "# Aliases":
            start = i
        elif line.strip() == "# End of Aliases":
            end = i
    if start is not None and end is not None:
        for line in lines[start + 1:end]:
            if line.strip().startswith("alias"):
                aliases.append(line.strip())
    else:
        print("No alias section found.")
        exit(1)

# Function to display the aliases
def display_aliases():
    table = Table(title="Aliases")
    table.add_column("Index", justify="right", style="cyan", no_wrap=True)
    table.add_column("Alias", style="magenta")
    for idx, alias in enumerate(aliases):
        table.add_row(str(idx), alias)
    console.print(table)

# Function to add an alias
def add_alias():
    name = Prompt.ask("Enter alias name")
    command = Prompt.ask("Enter alias command")
    new_alias = f"alias {name}='{command}'"
    aliases.append(new_alias)
    save_aliases()

# Function to edit an alias
def edit_alias():
    display_aliases()  # Display aliases before asking for index
    index = Prompt.ask("Enter alias index to edit")
    if index.isdigit():
        index = int(index)
        if 0 <= index < len(aliases):
            name = Prompt.ask("Enter new alias name")
            command = Prompt.ask("Enter new alias command")
            aliases[index] = f"alias {name}='{command}'"
            save_aliases()
        else:
            console.print("Invalid index", style="bold red")
    else:
        console.print("Please enter a valid number", style="bold red")

# Function to delete an alias
def delete_alias():
    display_aliases()  # Display aliases before asking for index
    index = Prompt.ask("Enter alias index to delete")
    if index.isdigit():
        index = int(index)
        if 0 <= index < len(aliases):
            del aliases[index]
            save_aliases()
        else:
            console.print("Invalid index", style="bold red")
    else:
        console.print("Please enter a valid number", style="bold red")

# Function to save aliases to the zshrc file
def save_aliases():
    with open(zshrc_path, 'r') as file:
        lines = file.readlines()
    start, end = None, None
    for i, line in enumerate(lines):
        if line.strip() == "# Aliases":
            start = i
        elif line.strip() == "# End of Aliases":
            end = i
    if start is not None and end is not None:
        with open(zshrc_path, 'w') as file:
            file.writelines(lines[:start + 1])
            for alias in aliases:
                file.write(alias + '\n')
            file.writelines(lines[end:])
        # Reload aliases only by spawning a new shell and sourcing aliases section only
        os.system(f'zsh -c "source <(grep -E \'^alias \' {zshrc_path})"')
        console.print("Aliases saved and .zshrc reloaded", style="bold green")
    else:
        console.print("No alias section found.", style="bold red")

# Initialize console
console = Console()

# Load aliases
load_aliases()

# Main loop
while True:
    console.print("\n1. View Aliases\n2. Add Alias\n3. Edit Alias\n4. Delete Alias\n5. Exit")
    choice = Prompt.ask("Enter your choice")
    if choice == '1':
        display_aliases()
    elif choice == '2':
        add_alias()
    elif choice == '3':
        edit_alias()
    elif choice == '4':
        delete_alias()
    elif choice == '5':
        break
    else:
        console.print("Invalid choice", style="bold red")

