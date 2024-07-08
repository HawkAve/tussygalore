package main

import (
	"fmt"
	"os"
	"os/exec"
	"strings"

	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

type item struct {
	title, desc string
}

func (i item) Title() string       { return i.title }
func (i item) Description() string { return i.desc }
func (i item) FilterValue() string { return i.title }

type model struct {
	list    list.Model
	choices []item
}

func getPyenvVersions() ([]string, error) {
	cmd := exec.Command("pyenv", "versions", "--bare")
	out, err := cmd.Output()
	if err != nil {
		return nil, err
	}
	return strings.Split(strings.TrimSpace(string(out)), "\n"), nil
}

func getHomebrewVersions() ([]string, error) {
	files, err := os.ReadDir("/opt/homebrew/opt")
	if err != nil {
		return nil, err
	}
	var versions []string
	for _, file := range files {
		if strings.HasPrefix(file.Name(), "python@") {
			versions = append(versions, file.Name())
		}
	}
	return versions, nil
}

func getInstalledPackages(version string, isPyenv bool) string {
	var cmd *exec.Cmd
	if isPyenv {
		cmd = exec.Command("pyenv", "exec", "python", "-m", "pip", "list")
	} else {
		cmd = exec.Command("/opt/homebrew/opt/"+version+"/bin/python3", "-m", "pip", "list")
	}
	out, err := cmd.Output()
	if err != nil {
		return "Error retrieving packages"
	}
	return string(out)
}

func switchVersion(version string) string {
	if strings.HasPrefix(version, "pyenv") {
		ver := strings.Split(version, " ")[1]
		cmd := exec.Command("pyenv", "global", ver)
		err := cmd.Run()
		if err != nil {
			return "Error switching pyenv version"
		}
		return "Switched to pyenv Python " + ver
	} else {
		ver := version
		binPath := "/opt/homebrew/opt/" + ver + "/bin"
		os.Setenv("PATH", binPath+":"+os.Getenv("PATH"))
		return "Switched to Homebrew Python " + ver
	}
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c", "q":
			return m, tea.Quit
		case "enter":
			selected := m.choices[m.list.Index()]
			var packages string
			if strings.HasPrefix(selected.Title(), "pyenv") {
				packages = getInstalledPackages(strings.Split(selected.Title(), " ")[1], true)
			} else {
				packages = getInstalledPackages(selected.Title(), false)
			}
			// Clear terminal
			fmt.Print("\033[H\033[2J")
			fmt.Println("Packages for", selected.Title(), ":\n")
			fmt.Println(packages)
			fmt.Println("\n-------------------------------------------------\n")
			fmt.Println(switchVersion(selected.Title()))
			return m, tea.Quit
		}
	}
	var cmd tea.Cmd
	m.list, cmd = m.list.Update(msg)
	return m, cmd
}

func (m model) View() string {
	return lipgloss.JoinHorizontal(lipgloss.Top, m.list.View())
}

func main() {
	pyenvVersions, err := getPyenvVersions()
	if err != nil {
		fmt.Printf("Error retrieving pyenv versions: %v\n", err)
		os.Exit(1)
	}
	homebrewVersions, err := getHomebrewVersions()
	if err != nil {
		fmt.Printf("Error retrieving Homebrew versions: %v\n", err)
		os.Exit(1)
	}

	var items []list.Item
	for _, v := range pyenvVersions {
		items = append(items, item{title: "pyenv " + v, desc: "Pyenv version"})
	}
	for _, v := range homebrewVersions {
		items = append(items, item{title: v, desc: "Homebrew version"})
	}

	delegate := list.NewDefaultDelegate()
	l := list.New(items, delegate, 30, 15)
	l.Title = "Select a Python version"

	m := model{
		list:    l,
		choices: make([]item, len(items)),
	}
	for i := range items {
		m.choices[i] = items[i].(item)
	}

	p := tea.NewProgram(m)
	if err := p.Start(); err != nil {
		fmt.Printf("Error: %v\n", err)
		os.Exit(1)
	}
}
