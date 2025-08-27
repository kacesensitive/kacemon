# kacemon

A cross-platform system resource monitor with a beautiful terminal interface.

## Features

- **Real-time system monitoring** - CPU, memory, network, temperature
- **Beautiful TUI** - Clean, colorful terminal interface
- **Network activity** - Live interface monitoring with visual bars
- **Temperature monitoring** - Hardware sensor readings in Fahrenheit
- **Process management** - Sortable process list with filtering
- **Cross-platform** - Works on Linux, macOS, and Windows

## Installation

```bash
cargo install kacemon
```

## Usage

```bash
# Run with default settings
kacemon

# Custom refresh rate (milliseconds)
kacemon --refresh 1000

# Light theme
kacemon --theme light

# Disable colors
kacemon --no-color
```

## Controls

- `q` - Quit
- `↑↓` - Navigate process list
- `s` - Sort processes
- `/` - Filter processes
- `?` - Help

## License

MIT
