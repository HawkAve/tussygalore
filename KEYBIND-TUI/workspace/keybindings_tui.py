import urwid

# Sample data for programs and keybindings
programs = ['skhdrc', 'yazi', '.zshrc', 'tmux', 'nvim']
keybindings = {
    'skhdrc': ['cmd + space: open spotlight', 'cmd + c: copy'],
    'yazi': ['a: add file', 'd: delete file'],
    '.zshrc': ['alias ll="ls -la"', 'export PATH=$PATH:/usr/local/bin'],
    'tmux': ['ctrl + b: prefix', 'ctrl + b + c: new window'],
    'nvim': [':w: save file', ':q: quit']
}

# Function to display keybindings for the selected program
def display_keybindings(program):
    return urwid.Text('\n'.join(keybindings.get(program, [])))

# ListBox for programs
program_list = urwid.ListBox(urwid.SimpleFocusListWalker([urwid.Text(p) for p in programs]))

# Placeholder for keybindings display
keybindings_display = urwid.Text('Select a program to view keybindings')

# Columns to hold the program list and keybindings display
columns = urwid.Columns([
    ('weight', 1, program_list),
    ('weight', 2, urwid.Filler(keybindings_display, valign='top'))
], dividechars=1)

# Function to add a keybinding
def add_keybinding(program, keybinding):
    if program in keybindings:
        keybindings[program].append(keybinding)
    else:
        keybindings[program] = [keybinding]

# Function to delete a keybinding
def delete_keybinding(program, keybinding):
    if program in keybindings and keybinding in keybindings[program]:
        keybindings[program].remove(keybinding)

# Function to search for a keybinding
def search_keybinding():
    search_edit = urwid.Edit("Search for a keybinding: ")
    loop = urwid.MainLoop(urwid.Filler(search_edit))
    loop.run()
    search_term = search_edit.get_edit_text()
    results = []
    for program, bindings in keybindings.items():
        for binding in bindings:
            if search_term in binding:
                results.append(f"{program}: {binding}")
    return urwid.Text('\n'.join(results))

# Main loop
def main():
    def on_program_change(widget, new_focus):
        program = programs[new_focus]
        keybindings_display.set_text('\n'.join(keybindings.get(program, [])))

    def handle_input(key):
        if key == 'a':
            program = programs[program_list.focus_position]
            keybinding_edit = urwid.Edit("Enter new keybinding: ")
            urwid.connect_signal(keybinding_edit, 'change', lambda edit, new_text: add_keybinding(program, new_text))
            urwid.MainLoop(urwid.Filler(keybinding_edit)).run()
            keybindings_display.set_text('\n'.join(keybindings.get(program, [])))
        elif key == 'd':
            program = programs[program_list.focus_position]
            keybinding_edit = urwid.Edit("Enter keybinding to delete: ")
            urwid.connect_signal(keybinding_edit, 'change', lambda edit, new_text: delete_keybinding(program, new_text))
            urwid.MainLoop(urwid.Filler(keybinding_edit)).run()
            keybindings_display.set_text('\n'.join(keybindings.get(program, [])))
        elif key == 's':
            search_results = search_keybinding()
            urwid.MainLoop(urwid.Filler(search_results)).run()
        elif key == 'q':
            raise urwid.ExitMainLoop()

    urwid.connect_signal(program_list.body, 'modified', on_program_change)
    urwid.MainLoop(columns, unhandled_input=handle_input).run()

if __name__ == '__main__':
    main()

