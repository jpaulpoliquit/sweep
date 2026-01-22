Title: Cleaning stalls and loading indicator freezes (investigation spec)

Owner: TBD
Status: Draft
Date: 2026-01-22

Summary
-------
Users report that cleaning can get stuck for a long time and eventually fail.
They also report the loading indicator (spinner/progress) freezing while work
is still happening. This spec documents the current code paths, likely causes,
and a concrete investigation and mitigation plan.

Scope
-----
This spec focuses on:
- TUI cleaning progress and animation stalls
- CLI cleaning delays where progress appears stuck
- Windows-specific cleanup paths (Recycle Bin, Windows Update, Event Logs)

Non-goals
---------
- Redesign of the scan pipeline
- New UI features unrelated to cleaning progress

Observed behavior (from code)
-----------------------------
TUI cleaning runs synchronously inside the main event loop:
- perform_cleanup(...) in src/tui/mod.rs performs all deletions in-process.
- It only increments app_state.tick and redraws between operations.
- Long blocking calls (trash_ops::delete_all, trash_ops::purge_all, net stop)
  prevent redraws, so the spinner appears frozen.

CLI cleaning uses indicatif progress bars:
- cleaner::clean_all(...) creates a progress bar with enable_steady_tick.
- Progress is only incremented after batch operations complete, so long batch
  deletes can look "stuck" even if the spinner is ticking in the background.

Key code paths
--------------
TUI:
- src/tui/mod.rs: perform_cleanup(...)
  - categories::trash::clean() -> trash_ops::purge_all (blocking)
  - cleaner::clean_paths_batch(...) for batch deletion (blocking)
  - windows update / event logs are not special-cased in TUI; they flow into
    batch deletion and may use Recycle Bin operations.

CLI:
- src/cli/commands/clean_command.rs -> cleaner::clean_all(...)
- src/cleaner/category_cleaning.rs -> clean_paths_batch(...)
- src/cleaner/batch_deletion.rs -> trash_ops::delete_all(...)

Windows-specific:
- src/categories/windows_update.rs -> clean_windows_update_downloads()
  uses `net stop wuauserv` and `net start wuauserv` with blocking Command::output.
- src/categories/trash.rs -> trash_ops::purge_all(...)

Likely root causes (hypotheses)
-------------------------------
H1. Long or blocking Recycle Bin operations freeze the TUI spinner.
    - trash_ops::delete_all and trash_ops::purge_all call Shell APIs.
    - These can block for a long time on large batches, slow disks, or network
      paths, and can show OS dialogs (file too large for Recycle Bin, access
      denied). Because perform_cleanup is synchronous, the UI cannot redraw.

H2. Batch deletion progress only updates after completion.
    - clean_paths_batch performs large deletes and only updates progress after
      returning. Users see no progress during the batch and assume a hang.

H3. Windows Update cleanup can hang on service stop/start.
    - clean_windows_update_downloads calls `net stop wuauserv` and waits. If
      the service is busy or blocked, the call can hang indefinitely.

H4. Fallback paths still use Recycle Bin operations.
    - If delete_all fails, the fallback still uses trash_ops::delete for each
      item. If the underlying cause is a UI prompt or COM conflict, this can
      repeatedly block or fail without visible progress.

H5. TUI animation depends on tick updates in the main loop.
    - During synchronous cleanup calls, tick does not advance, so the loading
      indicator freezes even if cleanup continues.

Investigation plan
------------------
1) Instrument long-running operations (dev logging)
   - Add timing logs (start/end + duration) around:
     - trash_ops::delete_all
     - trash_ops::purge_all
     - clean_windows_update_downloads (net stop/start)
     - clean_paths_batch overall and per batch
   - Log batch size, category, and the first path in the batch.
   - Gate logs behind a debug flag/env (e.g., WOLE_DEBUG_CLEAN=1).

2) Repro matrix (Windows)
   - Clean a large batch (10k+ files) in TUI, observe spinner freeze.
   - Clean files that exceed Recycle Bin size (forces OS prompt).
   - Clean paths on a network share or OneDrive (known slow Shell operations).
   - Clean Windows Update cache to exercise net stop/start.

3) Verify CLI behavior
   - Confirm whether indicatif spinner remains animated during long deletes.
   - If not, identify any stdout/stderr contention blocking the tick thread.

Mitigation plan (candidate changes)
-----------------------------------
A) Move TUI cleanup to a background worker thread
   - Similar to scan: spawn worker, send progress events via channel.
   - Main TUI loop continues to tick and redraw, keeping spinner alive.
   - Allows cancellation and avoids "frozen" UI perception.

B) Reduce perceived stalls with smaller batches and progress events
   - Split batch deletion into smaller chunks (e.g., 10-25) when using trash.
   - Emit progress after each chunk, even if the operation succeeds quickly.
   - Add an explicit "busy" state with last-progress timestamp.

C) Add timeouts for Windows service operations
   - Execute `net stop wuauserv` in a worker with a timeout and explicit
     messaging if it exceeds a threshold.

D) Detect and message Recycle Bin prompts
   - If delete_all returns an error that indicates a UI prompt or COM conflict,
     show a clear message and recommend --permanent or excluding those paths.

E) Provide fallback mode for known slow categories
   - For categories with known prompts (Downloads, Large, Duplicates),
     consider defaulting to smaller batches or one-by-one with progress.

Success criteria
----------------
- TUI spinner continues to animate during long clean operations.
- Users see regular progress updates even for large batches.
- Long-running operations log timings to identify bottlenecks.
- Windows Update cleanup does not hang indefinitely (timeout + message).

Open questions / data needed
----------------------------
- Which categories are most often reported as "stuck" by users?
- Are stalls observed on CLI, TUI, or both?
- Does the stall correlate with Recycle Bin prompts (file too large)?
- Are there specific file types or paths (OneDrive, network drives)?

Notes
-----
This spec is derived from current code paths in:
- src/tui/mod.rs
- src/cleaner/batch_deletion.rs
- src/cleaner/category_cleaning.rs
- src/categories/windows_update.rs
- src/categories/trash.rs
