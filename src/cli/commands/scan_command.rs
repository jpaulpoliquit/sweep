//! Scan command feature.
//!
//! This module owns and handles the "wole scan" command behavior.

use crate::cli::ScanOptions;
use crate::config::Config;
use crate::output::{self, OutputMode};
use crate::scanner;
use crate::size;
use std::path::PathBuf;

pub(crate) fn handle_scan(
    all: bool,
    cache: bool,
    app_cache: bool,
    temp: bool,
    trash: bool,
    build: bool,
    downloads: bool,
    large: bool,
    old: bool,
    applications: bool,
    windows_update: bool,
    event_logs: bool,
    path: Option<PathBuf>,
    json: bool,
    project_age: u64,
    min_age: u64,
    min_size: String,
    exclude: Vec<String>,
    force_full: bool,
    no_cache: bool,
    clear_cache: bool,
    output_mode: OutputMode,
) -> anyhow::Result<()> {
    // --all enables all categories
    let (
        cache,
        app_cache,
        temp,
        trash,
        build,
        downloads,
        large,
        old,
        applications,
        browser,
        system,
        empty,
        duplicates,
        windows_update,
        event_logs,
    ) = if all {
        (
            true, true, true, true, true, true, true, true, true, true, true, true, true, true,
            true,
        )
    } else if !cache
        && !app_cache
        && !temp
        && !trash
        && !build
        && !downloads
        && !large
        && !old
        && !applications
        && !windows_update
        && !event_logs
    {
        // No categories specified - show help message
        eprintln!("No categories specified. Use --all or specify categories like --cache, --app-cache, --temp, --build");
        eprintln!("Run 'wole scan --help' for more information.");
        return Ok(());
    } else {
        // Scan command doesn't support browser, system, empty, duplicates
        (
            cache,
            app_cache,
            temp,
            trash,
            build,
            downloads,
            large,
            old,
            applications,
            false,
            false,
            false,
            false,
            windows_update,
            event_logs,
        )
    };

    // Default to current directory to avoid stack overflow from OneDrive/UserDirs
    // PERFORMANCE FIX: Avoid OneDrive paths which are very slow to scan on Windows
    // Use current directory instead, which is faster and more predictable
    let scan_path = path.unwrap_or_else(|| {
        // Use current directory as default - faster and avoids OneDrive sync issues
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

    // Load config first
    let mut config = Config::load();

    // Apply CLI overrides to config
    config.apply_cli_overrides(
        Some(project_age),
        Some(min_age),
        Some(
            size::parse_size(&min_size).map_err(|e| {
                anyhow::anyhow!("Invalid size format '{}': {}", min_size, e)
            })? / (1024 * 1024),
        ), // Convert bytes to MB for config
    );

    // Merge CLI exclusions
    config.exclusions.patterns.extend(exclude.iter().cloned());

    // Handle cache flags
    let use_cache = !no_cache && config.cache.enabled && !force_full;
    
    if clear_cache {
        if let Ok(mut scan_cache) = crate::scan_cache::ScanCache::open() {
            // Get categories to clear
            let categories: Vec<&str> = if all {
                vec!["cache", "app_cache", "temp", "trash", "build", "downloads", "large", "old", "applications", "windows_update", "event_logs"]
            } else {
                let mut cats = Vec::new();
                if cache { cats.push("cache"); }
                if app_cache { cats.push("app_cache"); }
                if temp { cats.push("temp"); }
                if trash { cats.push("trash"); }
                if build { cats.push("build"); }
                if downloads { cats.push("downloads"); }
                if large { cats.push("large"); }
                if old { cats.push("old"); }
                if applications { cats.push("applications"); }
                if windows_update { cats.push("windows_update"); }
                if event_logs { cats.push("event_logs"); }
                cats
            };
            
            if categories.is_empty() {
                scan_cache.invalidate(None)?;
            } else {
                scan_cache.invalidate(Some(&categories))?;
            }
            
            if output_mode != OutputMode::Quiet {
                println!("Cache cleared for specified categories.");
            }
        }
    }

    // Use config values (after CLI overrides) for scan options
    let min_size_bytes = config.thresholds.min_size_mb * 1024 * 1024;

    let scan_options = ScanOptions {
        cache,
        app_cache,
        temp,
        trash,
        build,
        downloads,
        large,
        old,
        applications,
        browser,
        system,
        empty,
        duplicates,
        windows_update,
        event_logs,
        project_age_days: config.thresholds.project_age_days,
        min_age_days: config.thresholds.min_age_days,
        min_size_bytes,
    };

    // Open scan cache if enabled
    let mut scan_cache = if use_cache {
        match crate::scan_cache::ScanCache::open() {
            Ok(cache) => Some(cache),
            Err(e) => {
                if output_mode != OutputMode::Quiet {
                    eprintln!("Warning: Failed to open scan cache: {}. Continuing without cache.", e);
                }
                None
            }
        }
    } else {
        None
    };

    let results = scanner::scan_all(
        &scan_path,
        scan_options.clone(),
        output_mode,
        &config,
        scan_cache.as_mut(),
    )?;

    if json {
        output::print_json(&results)?;
    } else {
        output::print_human_with_options(&results, output_mode, Some(&scan_options));
    }

    Ok(())
}
