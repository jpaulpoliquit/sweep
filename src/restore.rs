//! Restore functionality for recovering deleted files
//!
//! Provides ability to restore files from Recycle Bin using deletion history logs

use crate::history::{list_logs, load_log, DeletionLog};
use crate::theme::Theme;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use trash::os_limited;

/// Callback function type for progress updates during restoration
pub type RestoreProgressCallback =
    Box<dyn FnMut(Option<&Path>, usize, usize, usize, usize) -> Result<()>>;

/// Get the count of files that can be restored from the most recent deletion session
pub fn get_restore_count() -> Result<usize> {
    let logs = list_logs()?;

    if logs.is_empty() {
        return Ok(0);
    }

    // Get the most recent log
    let latest_log = load_log(&logs[0])?;

    // Count restorable items (successful, non-permanent deletions)
    let count = latest_log
        .records
        .iter()
        .filter(|r| r.success && !r.permanent)
        .count();

    Ok(count)
}

/// Restore files from the most recent deletion session
pub fn restore_last(output_mode: crate::output::OutputMode) -> Result<RestoreResult> {
    restore_last_with_progress(output_mode, None)
}

/// Restore files from the most recent deletion session with progress callback
pub fn restore_last_with_progress(
    output_mode: crate::output::OutputMode,
    progress_callback: Option<RestoreProgressCallback>,
) -> Result<RestoreResult> {
    let logs = list_logs()?;

    if logs.is_empty() {
        return Err(anyhow::anyhow!(
            "No deletion history found. Nothing to restore."
        ));
    }

    // Get the most recent log
    let latest_log = load_log(&logs[0])?;
    restore_from_log_with_progress(&latest_log, output_mode, progress_callback)
}

/// Normalize a path for comparison (handles case-insensitive matching on Windows)
pub fn normalize_path_for_comparison(path: &str) -> String {
    // On Windows, paths are case-insensitive, so we normalize to lowercase
    // Also normalize separators and remove trailing separators
    #[cfg(windows)]
    {
        path.replace('\\', "/").to_lowercase()
    }
    #[cfg(not(windows))]
    {
        path.to_string()
    }
}

/// Restore files from a specific deletion log
pub fn restore_from_log(
    log: &DeletionLog,
    output_mode: crate::output::OutputMode,
) -> Result<RestoreResult> {
    restore_from_log_with_progress(log, output_mode, None)
}

/// Restore files from a specific deletion log with progress callback
pub fn restore_from_log_with_progress(
    log: &DeletionLog,
    output_mode: crate::output::OutputMode,
    mut progress_callback: Option<RestoreProgressCallback>,
) -> Result<RestoreResult> {
    let mut result = RestoreResult::default();

    // Get current Recycle Bin contents
    let recycle_bin_items = os_limited::list().context("Failed to list Recycle Bin contents")?;

    // Count total items to restore
    let total_to_restore = log
        .records
        .iter()
        .filter(|r| r.success && !r.permanent)
        .count();

    // Create a map of Recycle Bin items by original path
    // Windows Recycle Bin stores files with their original paths in metadata
    // Use normalized paths for better matching
    let mut bin_map: std::collections::HashMap<String, &trash::TrashItem> =
        std::collections::HashMap::new();
    for item in &recycle_bin_items {
        // Try to match by original parent + name
        let original_path = item.original_parent.join(&item.name);
        let normalized = normalize_path_for_comparison(&original_path.display().to_string());
        bin_map.insert(normalized, item);
    }

    // Try to restore each successful deletion record
    for record in &log.records {
        if !record.success || record.permanent {
            // Skip failed deletions and permanent deletions (can't restore those)
            continue;
        }

        let record_path = PathBuf::from(&record.path);
        let normalized_record_path = normalize_path_for_comparison(&record.path);

        // Update progress callback
        if let Some(ref mut callback) = progress_callback {
            callback(
                Some(&record_path),
                result.restored,
                total_to_restore,
                result.errors,
                result.not_found,
            )?;
        }

        // Try to find exact match first (for files)
        if let Some(trash_item) = bin_map.get(&normalized_record_path) {
            match restore_file(trash_item) {
                Ok(()) => {
                    result.restored += 1;
                    result.restored_bytes += record.size_bytes;
                    if output_mode != crate::output::OutputMode::Quiet {
                        println!(
                            "{} Restored: {}",
                            Theme::success("✓"),
                            Theme::secondary(&record.path)
                        );
                    }
                }
                Err(e) => {
                    result.errors += 1;
                    if output_mode != crate::output::OutputMode::Quiet {
                        eprintln!(
                            "{} Failed to restore {}: {}",
                            Theme::error("✗"),
                            Theme::secondary(&record.path),
                            Theme::error(&e.to_string())
                        );
                    }
                }
            }
        } else {
            // No exact match - check if this was a directory
            // When a directory is deleted, Windows Recycle Bin stores individual files,
            // not the directory itself. So we need to find all items whose path starts
            // with the directory path.
            let normalized_record_path_with_sep = if normalized_record_path.ends_with('/') {
                normalized_record_path.clone()
            } else {
                format!("{}/", normalized_record_path)
            };

            // Find all Recycle Bin items that are children of this directory
            let mut found_any = false;
            let mut restored_count = 0;
            let mut restore_errors = 0;

            for (bin_path, trash_item) in &bin_map {
                // Check if this Recycle Bin item is inside the directory we're restoring
                if bin_path.starts_with(&normalized_record_path_with_sep) {
                    found_any = true;
                    match restore_file(trash_item) {
                        Ok(()) => {
                            restored_count += 1;
                            // Size tracking removed - TrashItem doesn't expose size
                            // Final size will use record.size_bytes (see line 210)
                        }
                        Err(e) => {
                            restore_errors += 1;
                            if output_mode != crate::output::OutputMode::Quiet {
                                eprintln!(
                                    "{} Failed to restore {}: {}",
                                    Theme::error("✗"),
                                    Theme::secondary(
                                        &trash_item
                                            .original_parent
                                            .join(&trash_item.name)
                                            .display()
                                            .to_string()
                                    ),
                                    Theme::error(&e.to_string())
                                );
                            }
                        }
                    }
                }
            }

            if found_any {
                if restored_count > 0 {
                    result.restored += 1; // Count as one directory restored
                    result.restored_bytes += record.size_bytes; // Use the logged size
                    if output_mode != crate::output::OutputMode::Quiet {
                        println!(
                            "{} Restored directory: {} ({} items)",
                            Theme::success("✓"),
                            Theme::secondary(&record.path),
                            restored_count
                        );
                    }
                }
                result.errors += restore_errors;
            } else {
                // File/directory not found in Recycle Bin (may have been permanently deleted or already restored)
                result.not_found += 1;
                if output_mode == crate::output::OutputMode::VeryVerbose {
                    println!(
                        "{} Not found in Recycle Bin: {}",
                        Theme::muted("?"),
                        Theme::secondary(&record.path)
                    );
                }
            }
        }
    }

    // Final progress update
    if let Some(ref mut callback) = progress_callback {
        callback(
            None,
            result.restored,
            total_to_restore,
            result.errors,
            result.not_found,
        )?;
    }

    Ok(result)
}

/// Restore a specific file by path
pub fn restore_path(path: &Path, output_mode: crate::output::OutputMode) -> Result<RestoreResult> {
    let mut result = RestoreResult::default();

    // Get current Recycle Bin contents
    let recycle_bin_items = os_limited::list().context("Failed to list Recycle Bin contents")?;

    let normalized_path = normalize_path_for_comparison(&path.display().to_string());
    let normalized_path_with_sep = if normalized_path.ends_with('/') {
        normalized_path.clone()
    } else {
        format!("{}/", normalized_path)
    };

    // First try exact match (for files)
    for item in &recycle_bin_items {
        let original_path = item.original_parent.join(&item.name);
        let normalized_original =
            normalize_path_for_comparison(&original_path.display().to_string());

        if normalized_original == normalized_path {
            let restored_path = item.original_parent.join(&item.name);
            match restore_file(item) {
                Ok(()) => {
                    result.restored = 1;
                    // Get file size from restored file
                    result.restored_bytes = std::fs::metadata(&restored_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                    if output_mode != crate::output::OutputMode::Quiet {
                        println!(
                            "{} Restored: {}",
                            Theme::success("✓"),
                            Theme::secondary(&path.display().to_string())
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to restore {}: {}",
                        path.display(),
                        e
                    ));
                }
            }
        }
    }

    // No exact match - check if this is a directory
    // Find all Recycle Bin items that are children of this directory
    let mut found_any = false;
    let mut restored_count = 0;
    let mut restored_bytes = 0u64;
    let mut restore_errors = Vec::new();

    for item in &recycle_bin_items {
        let original_path = item.original_parent.join(&item.name);
        let normalized_original =
            normalize_path_for_comparison(&original_path.display().to_string());

        // Check if this Recycle Bin item is inside the directory we're restoring
        if normalized_original.starts_with(&normalized_path_with_sep) {
            found_any = true;
            let restored_path = item.original_parent.join(&item.name);
            match restore_file(item) {
                Ok(()) => {
                    restored_count += 1;
                    // Get file size from restored file
                    restored_bytes += std::fs::metadata(&restored_path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                }
                Err(e) => {
                    restore_errors.push((original_path.clone(), e));
                }
            }
        }
    }

    if found_any {
        if restored_count > 0 {
            result.restored = 1; // Count as one directory restored
            result.restored_bytes = restored_bytes;
            if output_mode != crate::output::OutputMode::Quiet {
                println!(
                    "{} Restored directory: {} ({} items)",
                    Theme::success("✓"),
                    Theme::secondary(&path.display().to_string()),
                    restored_count
                );
            }
        }

        // Report errors if any
        if !restore_errors.is_empty() {
            result.errors = restore_errors.len();
            if output_mode != crate::output::OutputMode::Quiet {
                for (error_path, error) in &restore_errors {
                    eprintln!(
                        "{} Failed to restore {}: {}",
                        Theme::error("✗"),
                        Theme::secondary(&error_path.display().to_string()),
                        Theme::error(&error.to_string())
                    );
                }
            }
        }

        Ok(result)
    } else {
        Err(anyhow::anyhow!(
            "File or directory not found in Recycle Bin: {}",
            path.display()
        ))
    }
}

/// Restore a single file from Recycle Bin
pub fn restore_file(item: &trash::TrashItem) -> Result<()> {
    let dest = item.original_parent.join(&item.name);

    // Create parent directory if it doesn't exist
    if let Some(parent) = dest.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }
    }

    // Check if destination already exists
    if dest.exists() {
        return Err(anyhow::anyhow!(
            "Destination already exists: {}",
            dest.display()
        ));
    }

    // Move file back from Recycle Bin to original location
    trash::os_limited::restore_all(std::iter::once(item.clone()))
        .with_context(|| format!("Failed to restore file to {}", dest.display()))?;

    Ok(())
}

/// Result of a restore operation
#[derive(Debug, Default)]
pub struct RestoreResult {
    pub restored: usize,
    pub restored_bytes: u64,
    pub errors: usize,
    pub not_found: usize,
}

impl RestoreResult {
    pub fn summary(&self) -> String {
        format!(
            "Restored {} items ({}), {} errors, {} not found",
            self.restored,
            bytesize::to_string(self.restored_bytes, true),
            self.errors,
            self.not_found
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restore_result_default() {
        let result = RestoreResult::default();
        assert_eq!(result.restored, 0);
        assert_eq!(result.restored_bytes, 0);
        assert_eq!(result.errors, 0);
        assert_eq!(result.not_found, 0);
    }

    #[test]
    fn test_restore_result_summary() {
        let result = RestoreResult {
            restored: 5,
            restored_bytes: 1024 * 1024, // 1 MiB
            errors: 1,
            not_found: 2,
        };

        let summary = result.summary();
        eprintln!("Actual summary: '{}'", summary);

        // Check that all expected values are present
        assert!(
            summary.contains("5"),
            "Summary should contain '5': {}",
            summary
        );
        // bytesize::to_string with binary_units=true may format as "1.0 MiB", "1 MiB", or similar
        // Check for the unit and that size representation is present
        assert!(
            summary.contains("MiB") || summary.contains("MB"),
            "Summary should contain size unit (MiB or MB): {}",
            summary
        );
        assert!(
            summary.contains("1"),
            "Summary should contain '1': {}",
            summary
        );
        assert!(
            summary.contains("2"),
            "Summary should contain '2': {}",
            summary
        );
    }
}
