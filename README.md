# Wole

*Deep clean and optimize your Windows PC.*

## Features

- **All-in-one toolkit**: CCleaner, WinDirStat, and TreeSize combined into a **single binary**
- **Deep cleaning**: Scans and removes caches, temp files, and browser leftovers to **reclaim gigabytes of space**
- **Project-aware**: Only cleans build artifacts from inactive projects (14+ days), respecting **Git status**
- **Disk insights**: Visualizes usage, finds large files, and explores your **disk space interactively**
- **Safe by default**: Dry-run mode, Recycle Bin deletion, and **full restore capability**

## Quick Start

**Install via PowerShell — recommended:**

```powershell
irm https://raw.githubusercontent.com/jpaulpoliquit/wole/master/install.ps1 | iex
```

**Or via Bash:**

```bash
curl -fsSL https://raw.githubusercontent.com/jpaulpoliquit/wole/master/install.sh | bash
```

**Run:**

```bash
wole                          # Interactive TUI menu (recommended!)
wole scan --all               # Preview what would be cleaned
wole clean --cache --temp     # Clean caches and temp files
wole clean --trash -y         # Empty Recycle Bin
wole analyze                  # Visual disk explorer
wole analyze --interactive    # Interactive disk insights TUI
wole restore --last           # Restore files from last deletion
wole restore --all            # Restore all Recycle Bin contents

wole config --show            # View current configuration
wole config --edit            # Edit config in your editor
wole remove                   # Uninstall wole from your system
wole remove --config --data   # Uninstall and remove all data
wole --help                   # Show help
wole --version                # Show installed version

wole scan --all -v            # Verbose scan with file paths
wole scan --all --json        # JSON output for scripting
wole clean --all --dry-run    # Preview cleanup without deleting
wole clean --all --permanent  # Bypass Recycle Bin (use with caution!)
wole status                   # Real-time system health dashboard
wole status --json            # Status output as JSON
wole optimize --all           # Run all system optimizations
wole update                   # Check for and install updates
```

## Tips

- **Terminal**: Works best with Windows Terminal, PowerShell, or any modern terminal emulator.
- **Safety**: Built with strict protections. See [Security Audit](SECURITY_AUDIT.md). Preview changes with `wole scan --all` or `--dry-run`.
- **Verbose Mode**: Use `-v` or `-vv` for detailed output showing file paths and scan progress.
- **Navigation**: TUI supports arrow keys for intuitive navigation.
- **Configuration**: Run `wole config --edit` to customize thresholds, exclusions, and scan paths.
- **System Monitoring**: Use `wole status` to monitor system health in real-time. The dashboard auto-refreshes every second.
- **System Optimization**: Run `wole optimize --all` to perform various Windows optimizations. Some operations require administrator privileges.

## Features in Detail

### Deep System Cleanup

```bash
$ wole clean --all

Scanning for cleanable files...
Found 362 files (8.1 GB) to clean.

Cleaning...
✓ Package cache: 45 files (2.3 GB)
✓ Temp: 128 files (456 MB)
✓ Trash: 23 files (89 MB)
✓ Build: 12 files (1.2 GB)
✓ Browser: 67 files (234 MB)
✓ System: 34 files (567 MB)

====================================================================
Space freed: 8.1 GB | Free space now: 53.3 GB
====================================================================
```

### Interactive TUI Mode

```bash
$ wole

┌─────────────────────────────────────────────────────────┐
│                                                         │
│    ██╗    ██╗ ██████╗ ██╗     ███████╗                  │
│    ██║    ██║██╔═══██╗██║     ██╔════╝                  │
│    ██║ █╗ ██║██║   ██║██║     █████╗                    │
│    ██║███╗██║██║   ██║██║     ██╔══╝                    │
│    ╚███╔███╔╝╚██████╔╝███████╗███████╗                  │
│     ╚══╝╚══╝  ╚═════╝ ╚══════╝╚══════╝                  │
│                                                         │
│    Windows-first cleanup tool                           │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ ACTIONS                                                 │
│                                                         │
│ > Scan        Find cleanable files (safe, dry-run)      │
│   Clean       Delete selected files                     │
│   Analyze     Explore disk usage (folder sizes)         │
│   Restore     Restore files from last deletion          │
│   Config      View or modify settings                   │
│   Optimize    Optimize Windows system performance       │
│   Status      Real-time system health dashboard         │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ Select categories to scan:                              │
│                                                         │
│ [X] Package cache    [X] Temp    [X] Trash              │
│ [X] Build            [ ] Browser [ ] System             │
│                                                         │
└─────────────────────────────────────────────────────────┘
[↑↓] Navigate  [Space] Toggle  [Enter] Confirm  [Esc] Exit
```

### Disk Space Analyzer

```bash
$ wole analyze

Disk Insights: C:\Users\user
Total: 120 GB │ 15,234 files

#  ████████████████████  100.0%  120 GB
   C:\Users\user

1  ████████████████░░░░   85.2%  102 GB  Projects
2  ████░░░░░░░░░░░░░░░░   18.5%   22 GB  Downloads
3  ██░░░░░░░░░░░░░░░░░░    8.3%   10 GB  Documents

Largest Files:
  45 GB  C:\Users\user\Projects\game\assets.bin
  12 GB  C:\Users\user\Downloads\movie.mkv

[↑↓] Navigate  [Enter] Open  [Esc] Back  [S] Sort
```

### Project-Aware Build Cleanup

Clean old build artifacts (`node_modules`, `target`, `bin/obj`, etc.) from inactive projects while respecting Git status.

```bash
$ wole clean --build

Scanning for cleanable files...

Build                12    1.2 GB    [OK] Inactive projects
  C:\Users\user\Projects\old-react-app\node_modules
  C:\Users\user\Projects\rust-experiment\target
  C:\Users\user\Projects\dotnet-api\bin
  ... and 9 more

====================================================================
Space freed: 1.2 GB
====================================================================
```

> **Smart detection:** Only cleans projects inactive for 14+ days. Skips projects with recent commits or uncommitted changes.

### Scan Results

```bash
$ wole scan --all

╔════════════════════════════════════════════════════════════╗
║                    Wole Scan Results                       ║
╠════════════════════════════════════════════════════════════╣

Category         Items      Size         Status
────────────────────────────────────────────────────────────
Package cache        45    2.3 GB    [OK] Safe to clean
Temp                128    456 MB     [OK] Safe to clean
Trash                23    89 MB      [OK] Safe to clean
Build                12    1.2 GB    [OK] Inactive projects
Browser              67    234 MB     [OK] Safe to clean
System               34    567 MB     [OK] Safe to clean
Large                 8    2.1 GB   [!] Review suggested
Old                  45    890 MB   [!] Review suggested
────────────────────────────────────────────────────────────
Total              362    8.1 GB         Reclaimable

Run wole clean --all to remove these files.
```

### File Restore

Easily restore files from your last deletion session or restore all Recycle Bin contents in bulk.

```bash
# Restore from last deletion session (uses bulk restore for better performance)
$ wole restore --last

# Restore all contents of Recycle Bin in bulk (fastest option on Windows)
$ wole restore --all

# Restore a specific file or directory
$ wole restore --path "C:\Users\user\Documents\file.txt"
```

Restore operations use bulk restore by default for better performance on Windows.

### System Status Dashboard

Monitor your system's health in real-time with comprehensive metrics.

```bash
$ wole status

┌─────────────────────────────────────────────────────────┐
│ Health status: ● 85  Live                               │
│ DESKTOP-ABC123 · Intel Core i7-9700K · 32.0GB · Windows │
├─────────────────────────────────────────────────────────┤
│ ⚙ CPU                    ▦ Memory                      │
│ Total   ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱  45.2%      │
│ Load    2.34 / 1.89 / 1.45 (8 cores)                    │
│                                                         │
│ ▤ Disk                    ⚡ Power                     │
│ Used    ▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▱▱▱▱  78.5%      │
│ Free    234.5 GB                                        │
│                                                         │
│ ⇅ Network                 ▶ Processes                  │
│ Down    ▰▰▰▱▱  12.3 MB/s                            │
│ Up      ▰▱▱▱▱   2.1 MB/s                            │
│ Status  Connected · WiFi                                │
│ IPv4    192.168.1.100                                   │
└─────────────────────────────────────────────────────────┘
```

**With Battery (Laptop):**

```bash
$ wole status

┌─────────────────────────────────────────────────────────┐
│ Health status: ● 85  Live                               │
│ LAPTOP-ABC123 · Intel Core i7-9700K · 32.0GB · Windows  │
├─────────────────────────────────────────────────────────┤
│ ⚙ CPU                                                   │
│ Total   ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱  45.2%      │
│ Load    2.34 / 1.89 / 1.45 (8 cores)                    │
│                                                         │
│ ▦ Memory                                                │
│ Used    ▰▰▰▰▰▰▰▰▱▱▱▱▱▱▱▱▱▱▱▱  45.2%      │
│ Total   24.5 / 32.0 GB                                   │
│                                                          │
│ ▤ Disk                                                   │
│ Used    ▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▰▱▱▱▱  78.5%       │
│ Free    234.5 GB                                         │
│ Read    ▰▰▰▱▱  45.2 MB/s                             │
│ Write   ▰▱▱▱▱  12.3 MB/s                             │
│                                                          │
│ ⚡ Power                                                │
│ Level   ▰▰▰▰▰▰▰▰▰▰▰▱▱  87.5%                   │
│ Status  Charging                                         │
│ Health  Good                                             │
│ Cycles  245                                              │
│ Time    2h 15m to full                                   │
│ Volt    12.45 V                                          │
│ Power   15.2 W                                           │
│ Design  85000 mWh                                        │
│ Full    82000 mWh                                        │
│                                                          │
│ ⇅ Network                                                │
│ Down    ▰▰▰▱▱  12.3 MB/s                             │
│ Up      ▰▱▱▱▱   2.1 MB/s                             │
│ Status  Connected · WiFi                                 │
│ IPv4    192.168.1.100                                    │
│                                                          │
│ ▶ Processes                                             │
│ chrome.exe        1234  ▰▰▰▱▱  15.2%  245M           │
│ code.exe          5678  ▰▰▱▱▱   8.5%  180M           │
└──────────────────────────────────────────────────────────┘
```

**Battery Information Displayed:**

- **Level**: Current battery percentage with visual progress bar
- **Status**: Charging, Discharging, Full, or Not Charging
- **Health**: Good (≥80%), Fair (≥50%), or Poor (<50%)
- **Cycles**: Number of charge/discharge cycles (if available)
- **Time**: Estimated time until empty or until fully charged
- **Voltage**: Current battery voltage in volts
- **Power**: Current power draw/charge rate in watts
- **Design Capacity**: Original battery capacity when new (mWh)
- **Full Charge Capacity**: Current maximum capacity (mWh)

On desktop systems without a battery, the Power section shows "Status  Plugged In".

The status dashboard shows:

- **Health Score**: Overall system health (0-100)
- **CPU**: Usage, load averages, core details, frequency, vendor info
- **Memory**: Used, total, free, swap/page file
- **Disk**: Usage, free space, read/write speeds
- **Power**: Battery level, status, health, cycles, temperature (laptops)
- **Network**: Download/upload speeds, connection status, IP addresses
- **Processes**: Top 10 processes by CPU usage

Use `wole status --json` for JSON output suitable for scripting.

## Commands

### Core Commands

- `scan` - Find cleanable files (safe, dry-run)
- `clean` - Delete selected files
- `analyze` - Explore disk usage or show detailed analysis
- `restore` - Restore files from deletion or Recycle Bin
- `config` - View or modify configuration
- `status` - Real-time system health dashboard
- `optimize` - Optimize Windows system performance
- `update` - Check for and install updates
- `remove` - Uninstall wole from your system

### Categories


| Flag             | Description                                                                         |
| ---------------- | ----------------------------------------------------------------------------------- |
| `--cache`        | Package manager caches (npm/yarn/pnpm, NuGet, Cargo, pip)                           |
| `--app-cache`    | Application caches (Discord, VS Code, Slack, Spotify)                               |
| `--temp`         | Windows temp files older than 1 day                                                 |
| `--trash`        | Recycle Bin contents                                                                |
| `--build`        | Build artifacts from inactive projects (`node_modules`, `target/`, `bin/obj`, etc.) |
| `--browser`      | Browser caches (Chrome, Edge, Firefox, Brave, etc.)                                 |
| `--system`       | Windows system caches (thumbnails, updates, icons)                                  |
| `--downloads`    | Old files in Downloads (30+ days)                                                   |
| `--large`        | Large files (100MB+)                                                                |
| `--old`          | Files not accessed in 30+ days                                                      |
| `--empty`        | Empty folders                                                                       |
| `--duplicates`   | Duplicate files                                                                     |
| `--applications` | Installed applications                                                              |


**Note:** Only `--build` is project-aware. Other categories clean files system-wide.

## Options

**Common:**

- `--all` - Enable all categories
- `--exclude <PATTERN>` - Exclude paths (repeatable)
- `--json` - JSON output for scripting
- `-v`, `-vv` - Verbose output
- `-q` - Quiet mode

**Scan:**

- `--project-age <DAYS>` - Project inactivity threshold for `--build` (default: 14)
- `--min-age <DAYS>` - Minimum file age for `--downloads` and `--old` (default: 30)
- `--min-size <SIZE>` - Minimum file size for `--large` (default: 100MB)

**Clean:**

- `-y`, `--yes` - Skip confirmation
- `--permanent` - Bypass Recycle Bin
- `--dry-run` - Preview only

**Status:**

- `--json` - Output as JSON for scripting
- `-w`, `--watch` - Continuous refresh mode (TUI auto-refreshes by default)

**Optimize:**

- `--all` - Run all optimizations
- `--dns` - Flush DNS cache
- `--thumbnails` - Clear thumbnail cache
- `--icons` - Rebuild icon cache and restart Explorer
- `--databases` - Optimize browser databases (VACUUM)
- `--fonts` - Restart font cache service (requires admin)
- `--memory` - Clear standby memory (requires admin)
- `--network` - Reset network stack (requires admin)
- `--bluetooth` - Restart Bluetooth service (requires admin)
- `--search` - Restart Windows Search service (requires admin)
- `--explorer` - Restart Windows Explorer
- `--dry-run` - Preview only
- `-y`, `--yes` - Skip confirmation for admin operations

## Configuration

Config file: `%APPDATA%\wole\config.toml`

```toml
[thresholds]
project_age_days = 14
min_age_days = 30
min_size_mb = 100

[exclusions]
patterns = ["**/important-project/**"]
```

```bash
wole config --show    # View config
wole config --edit    # Edit config
wole remove           # Uninstall wole
wole remove --config --data  # Uninstall and remove all data
```

## Building from Source

**Prerequisites:** Rust, Visual Studio Build Tools

```powershell
cargo build --release
```

**Output:** `target\release\wole.exe`

## Troubleshooting

- **File locked:** File is open in another app. Will be skipped automatically.
- **Long paths:** Handled automatically. Update if issues persist.
- **Symlinks:** Automatically skipped (expected behavior).
- **TUI not working:** Use PowerShell/Windows Terminal, or CLI mode: `wole scan --all`
- **No items found:** Check project activity with `--project-age 0` or file ages with `--min-age 0`

## Support

- If Wole saved you disk space, consider starring the repo or sharing it with friends.
- Have ideas or fixes? Check our [Contributing Guide](CONTRIBUTING.md), then open an issue or PR.
- Follow the author on [X (Twitter)](https://x.com/jpaulpoliquit) for updates!

## License

[MIT License](LICENSE) — feel free to enjoy and participate in open source