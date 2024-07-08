import os
from pathlib import Path
from textual.app import App, ComposeResult
from textual.containers import Container
from textual.widgets import Header, Footer, ListView, ListItem, Label, Static
from textual.events import Key

class PythonVersionSwitcher(App):

    CSS = """
    #versions {
        height: 1fr;
        overflow: auto;
    }
    #packages {
        height: 1fr;
        overflow: auto;
    }
    """

    def __init__(self):
        super().__init__()
        self.versions = []
        self.packages = []

    def get_pyenv_versions(self):
        result = os.popen("pyenv versions --bare").read()
        versions = result.split()
        return ["pyenv " + version for version in versions]

    def get_homebrew_versions(self):
        homebrew_opt_path = Path("/opt/homebrew/opt")
        versions = [f"homebrew {p.name}" for p in homebrew_opt_path.glob("python@*")]
        return versions

    def get_installed_packages(self, version, is_pyenv):
        if is_pyenv:
            python_bin = f"pyenv exec python -m pip list"
        else:
            homebrew_bin_path = f"/opt/homebrew/opt/{version}/bin"
            python_bin = f"{homebrew_bin_path}/python3 -m pip list"
        
        result = os.popen(python_bin).read()
        return result

    def compose(self) -> ComposeResult:
        self.header = Header()
        self.footer = Footer()
        self.footer_message = Label(id="footer_message")
        self.versions_list = ListView(id="versions")
        self.packages_view = Static(id="packages")

        yield Container(
            self.header,
            Container(
                self.versions_list,
                self.packages_view,
                id="content"
            ),
            self.footer,
            self.footer_message,
        )

    def on_mount(self) -> None:
        pyenv_versions = self.get_pyenv_versions()
        homebrew_versions = self.get_homebrew_versions()
        
        if not pyenv_versions and not homebrew_versions:
            self.footer_message.update("No Python versions found in pyenv or /opt/homebrew/opt")
            return

        self.versions = pyenv_versions + homebrew_versions

        for version in self.versions:
            self.versions_list.append(ListItem(Label(version)))

    def on_list_view_selected(self, message) -> None:
        selected_index = self.versions_list.index
        selected_version = self.versions[selected_index]
        if selected_version.startswith("pyenv"):
            version = selected_version.split()[1]
            packages = self.get_installed_packages(version, True)
        elif selected_version.startswith("homebrew"):
            version = selected_version.split()[1]
            packages = self.get_installed_packages(version, False)
        
        self.packages_view.update(packages)

    def switch_version(self, selected_version):
        if selected_version.startswith("pyenv"):
            version = selected_version.split()[1]
            os.system(f"pyenv global {version}")
            self.footer_message.update(f"Switched to pyenv Python {version}.")
        elif selected_version.startswith("homebrew"):
            version = selected_version.split()[1]
            homebrew_bin_path = f"/opt/homebrew/opt/{version}/bin"
            os.environ["PATH"] = f"{homebrew_bin_path}:{os.environ['PATH']}"
            self.footer_message.update(f"Switched to Homebrew Python {version}. Current PATH: {os.environ['PATH']}")

    async def on_key(self, event: Key) -> None:
        if event.key == "enter":
            selected_index = self.versions_list.index
            selected_version = self.versions[selected_index]
            self.switch_version(selected_version)

if __name__ == "__main__":
    PythonVersionSwitcher().run()

