use crate::categories;
use crate::cli::ScanOptions;
use crate::output::ScanResults;
use anyhow::{Context, Result};
use std::path::Path;

/// Scan all requested categories and return aggregated results
/// 
/// Handles errors gracefully - if one category fails, others continue scanning
pub fn scan_all(path: &Path, options: ScanOptions) -> Result<ScanResults> {
    let mut results = ScanResults::default();
    
    if options.cache {
        match categories::cache::scan(path) {
            Ok(cache_result) => results.cache = cache_result,
            Err(e) => eprintln!("Warning: Cache scan failed: {}", e),
        }
    }
    
    if options.temp {
        match categories::temp::scan(path) {
            Ok(temp_result) => results.temp = temp_result,
            Err(e) => eprintln!("Warning: Temp scan failed: {}", e),
        }
    }
    
    if options.trash {
        match categories::trash::scan() {
            Ok(trash_result) => results.trash = trash_result,
            Err(e) => eprintln!("Warning: Trash scan failed: {}", e),
        }
    }
    
    if options.build {
        match categories::build::scan(path, options.project_age_days) {
            Ok(build_result) => results.build = build_result,
            Err(e) => eprintln!("Warning: Build scan failed: {}", e),
        }
    }
    
    Ok(results)
}
