import subprocess
from rich.console import console
from rich.table import table
from rich.markdown import markdown

console = Console()

def get_python_version():
    result = subprocess.run(['python', 'versions', '--bare'], capture_output=True)
    versions = result.stdout.split()
    return versions


