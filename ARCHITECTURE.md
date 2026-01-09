# Sweeper Architecture Documentation

This document describes the purpose and responsibilities of each file in the sweeper codebase.

## Overview

Sweeper is a Windows-first developer cleanup tool that safely removes unused files and build artifacts from inactive projects. The architecture follows a modular design with clear separation of concerns.

## Core Files

### `src/main.rs`
**Purpose:** Application entry point  
**Responsibilities:**
- Initializes the CLI parser
- Delegates command execution to `Cli::run()`
- Handles top-level error propagation

**Key Functions:**
- `main()` - Entry point that parses CLI and runs commands

---

### `src/cli.rs`
**Purpose:** Command-line interface definition and command routing  
**Responsibilities:**
- Defines CLI structure using `clap` derive macros
- Parses command-line arguments
- Routes commands to appropriate handlers (scan/clean)
- Manages default values (home directory, project age threshold)

**Key Structures:**
- `Cli` - Main CLI parser struct
- `Commands` - Enum of available commands (Scan, Clean)
- `ScanOptions` - Options passed to scanner

**Key Functions:**
- `Cli::parse()` - Parse command-line arguments
- `Cli::run()` - Execute the parsed command

**Command Structure:**
- `scan` - Dry-run mode, finds cleanable files
- `clean` - Deletes files with confirmation prompt

---

### `src/scanner.rs`
**Purpose:** Orchestrates scanning across all categories  
**Responsibilities:**
- Coordinates scanning of multiple categories
- Aggregates results from all category scanners
- Handles errors gracefully (one category failure doesn't stop others)
- Returns unified `ScanResults` structure

**Key Functions:**
- `scan_all()` - Scans all requested categories and aggregates results

**Error Handling:**
- Individual category failures are logged as warnings but don't stop the scan
- Returns partial results if some categories succeed

---

### `src/cleaner.rs`
**Purpose:** Handles file deletion operations  
**Responsibilities:**
- Manages confirmation prompts (unless `-y` flag is used)
- Coordinates deletion across all categories
- Provides progress feedback during cleanup
- Tracks success/failure counts
- Moves files to Recycle Bin (safe deletion)

**Key Functions:**
- `clean_all()` - Deletes files based on scan results

**Safety Features:**
- Confirmation prompt by default
- Progress messages for each category
- Error tracking and reporting
- All deletions go to Recycle Bin

---

### `src/output.rs`
**Purpose:** Formats and displays scan results  
**Responsibilities:**
- Formats results for human-readable output (tables)
- Formats results as JSON for automation/CI
- Calculates totals across categories
- Provides size formatting (human-readable bytes)

**Key Structures:**
- `ScanResults` - Aggregated results from all categories
- `CategoryResult` - Results for a single category (items, size, paths)
- `JsonResults` - JSON serialization structure

**Key Functions:**
- `print_human()` - Pretty-printed table output
- `print_json()` - JSON output for automation
- `CategoryResult::size_human()` - Format bytes as KB/MB/GB

---

## Project Detection & Activity

### `src/project.rs`
**Purpose:** Detects projects and determines if they're active  
**Responsibilities:**
- Detects project type by marker files (package.json, Cargo.toml, etc.)
- Finds all project roots in a directory tree
- Determines if a project is "active" (recently modified or has uncommitted changes)
- Prevents deletion of active project artifacts

**Key Structures:**
- `ProjectType` - Enum of supported project types (Node, Rust, .NET, Python, Java)

**Key Functions:**
- `detect_project_type()` - Identifies project type by marker files
- `find_project_roots()` - Recursively finds all project roots
- `is_project_active()` - Checks if project has recent activity

**Activity Detection Logic:**
1. Checks git repository for uncommitted changes (dirty = active)
2. Checks git last commit date (recent = active)
3. Falls back to project file modification time

**Supported Project Types:**
- Node.js: `package.json`
- Rust: `Cargo.toml`
- .NET: `*.csproj`, `*.sln`
- Python: `pyproject.toml`, `requirements.txt`
- Java: `build.gradle`, `pom.xml`

---

### `src/git.rs`
**Purpose:** Git repository inspection utilities  
**Responsibilities:**
- Finds git root directory by walking up directory tree
- Checks if repository has uncommitted changes (dirty)
- Gets last commit timestamp
- Used by project activity detection

**Key Functions:**
- `find_git_root()` - Locates `.git` directory
- `is_dirty()` - Checks for uncommitted changes
- `last_commit_date()` - Gets timestamp of last commit

**Error Handling:**
- Returns `None`/`false` gracefully if not a git repo
- Doesn't fail if git operations can't complete

---

## Category Modules

### `src/categories/mod.rs`
**Purpose:** Module declaration for all category scanners  
**Responsibilities:**
- Exports all category modules (cache, temp, trash, build)

---

### `src/categories/cache.rs`
**Purpose:** Scans and cleans package manager caches  
**Responsibilities:**
- Detects npm, pip, and yarn cache directories
- Calculates total size of cache directories
- Deletes cache directories (moves to Recycle Bin)

**Scanned Locations:**
- `%LOCALAPPDATA%\npm-cache` (npm)
- `%LOCALAPPDATA%\pip\cache` (pip)
- `%LOCALAPPDATA%\Yarn\Cache` (yarn)

**Key Functions:**
- `scan()` - Finds cache directories and calculates sizes
- `clean()` - Deletes cache directory
- `calculate_size()` - Recursively calculates directory size

**Safety:**
- Only scans well-known cache locations
- Safe to delete (caches can be regenerated)

---

### `src/categories/temp.rs`
**Purpose:** Scans and cleans temporary files  
**Responsibilities:**
- Finds temporary files older than 1 day
- Scans Windows temp directories
- Deletes old temp files

**Scanned Locations:**
- `%TEMP%` directory
- `%LOCALAPPDATA%\Temp` directory

**Key Functions:**
- `scan()` - Finds temp files older than 1 day
- `scan_temp_dir()` - Helper to scan a specific temp directory
- `clean()` - Deletes temp file

**Safety:**
- Only deletes files older than 1 day
- Limits scan depth to 3 levels to avoid excessive scanning
- Skips symlinks

---

### `src/categories/trash.rs`
**Purpose:** Manages Windows Recycle Bin  
**Responsibilities:**
- Lists items in Recycle Bin
- Empties Recycle Bin
- Uses Windows-specific APIs via `trash` crate

**Key Functions:**
- `scan()` - Lists Recycle Bin items (count only, size not calculated)
- `clean()` - Empties entire Recycle Bin

**Note:**
- Size calculation is skipped (would require reading each file, expensive)
- Only item count is tracked

---

### `src/categories/build.rs`
**Purpose:** Scans and cleans build artifacts from inactive projects  
**Responsibilities:**
- Finds build artifact directories in projects
- Only targets inactive projects (via project activity detection)
- Calculates sizes of build artifacts
- Deletes build artifacts safely

**Detected Build Artifacts:**
- `node_modules/` (Node.js)
- `target/` (Rust)
- `bin/`, `obj/` (.NET)
- `dist/`, `build/` (general)
- `.next/`, `.nuxt/`, `.output/` (Next.js, Nuxt)
- `__pycache__/`, `.pytest_cache/`, `.mypy_cache/` (Python)
- `.venv/`, `venv/` (Python virtual environments)
- `.gradle/` (Java/Gradle)
- `.parcel-cache/`, `.turbo/` (build tools)

**Key Functions:**
- `scan()` - Finds build artifacts in inactive projects
- `find_build_artifacts()` - Locates artifact directories in a project
- `calculate_size()` - Recursively calculates directory size
- `clean()` - Deletes build artifact directory

**Safety:**
- Only targets inactive projects (no recent commits, no uncommitted changes)
- Uses project activity detection to avoid active work
- Skips symlinks

---

## Data Flow

### Scan Flow
```
main.rs → cli.rs → scanner.rs → categories/*.rs → output.rs
```

1. User runs `sweeper scan --cache --build`
2. `main.rs` parses CLI and calls `Cli::run()`
3. `cli.rs` creates `ScanOptions` and calls `scanner::scan_all()`
4. `scanner.rs` calls each requested category's `scan()` function
5. Categories scan filesystem and return `CategoryResult`
6. `scanner.rs` aggregates results into `ScanResults`
7. `cli.rs` calls `output::print_human()` or `output::print_json()`
8. Results displayed to user

### Clean Flow
```
main.rs → cli.rs → scanner.rs → output.rs → cleaner.rs → categories/*.rs
```

1. User runs `sweeper clean --cache --build -y`
2. Same scan flow as above
3. Results displayed
4. `cleaner.rs::clean_all()` called with results
5. Confirmation prompt (unless `-y` flag)
6. For each category with items, calls category's `clean()` function
7. Progress messages displayed
8. Summary with success/error counts

---

## Error Handling Strategy

### Principles
1. **Graceful Degradation**: One category failure doesn't stop others
2. **User-Friendly Messages**: Clear error messages with context
3. **Safe Defaults**: Errors default to skipping rather than failing
4. **Context Preservation**: Uses `anyhow::Context` for error chains

### Error Handling by Layer

**Category Level:**
- Individual file/directory errors logged as warnings
- Returns partial results if some items succeed

**Scanner Level:**
- Category failures logged but don't stop scan
- Returns partial `ScanResults` if some categories succeed

**Cleaner Level:**
- Individual deletion failures logged as warnings
- Continues cleaning other items
- Reports total success/error counts

---

## Safety Features

1. **Dry-Run by Default**: `scan` command never deletes
2. **Confirmation Prompts**: `clean` requires confirmation (unless `-y`)
3. **Recycle Bin**: All deletions go to Recycle Bin (recoverable)
4. **Active Project Protection**: Never deletes artifacts from active projects
5. **Age Thresholds**: Temp files must be older than 1 day
6. **Project Activity Detection**: Git-aware, file modification-aware

---

## Dependencies

- `clap` - CLI argument parsing
- `walkdir` - Directory traversal
- `trash` - Windows Recycle Bin integration
- `git2` - Git repository inspection
- `serde`/`serde_json` - JSON serialization
- `chrono` - Date/time handling
- `bytesize` - Human-readable size formatting
- `directories` - User directory detection
- `anyhow` - Error handling

---

## Future Extensibility

The architecture supports adding new categories easily:

1. Create new file in `src/categories/` (e.g., `downloads.rs`)
2. Implement `scan()` and `clean()` functions
3. Add module to `src/categories/mod.rs`
4. Add flag to `cli.rs` `ScanOptions`
5. Add category to `output.rs` `ScanResults`
6. Wire up in `scanner.rs` and `cleaner.rs`

This modular design makes Phase 2+ features straightforward to add.
