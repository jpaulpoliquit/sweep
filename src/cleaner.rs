use crate::categories;
use crate::output::ScanResults;
use anyhow::Result;
use std::io::{self, Write};

/// Clean all categories based on scan results
/// 
/// Handles confirmation prompts, error tracking, and provides progress feedback
pub fn clean_all(results: &ScanResults, skip_confirm: bool) -> Result<()> {
    let total_items = results.cache.items
        + results.temp.items
        + results.trash.items
        + results.build.items;
    
    if total_items == 0 {
        println!("Nothing to clean.");
        return Ok(());
    }
    
    if !skip_confirm {
        print!("Delete {} items? [y/N]: ", total_items);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }
    
    let mut cleaned = 0;
    let mut errors = 0;
    
    // Clean cache
    if results.cache.items > 0 {
        println!("Cleaning cache directories...");
        for path in &results.cache.paths {
            match categories::cache::clean(path) {
                Ok(()) => cleaned += 1,
                Err(e) => {
                    errors += 1;
                    eprintln!("Warning: Failed to clean {}: {}", path.display(), e);
                }
            }
        }
    }
    
    // Clean temp
    if results.temp.items > 0 {
        println!("Cleaning temp files...");
        for path in &results.temp.paths {
            match categories::temp::clean(path) {
                Ok(()) => cleaned += 1,
                Err(e) => {
                    errors += 1;
                    eprintln!("Warning: Failed to clean {}: {}", path.display(), e);
                }
            }
        }
    }
    
    // Clean trash
    if results.trash.items > 0 {
        println!("Emptying Recycle Bin...");
        match categories::trash::clean() {
            Ok(()) => cleaned += results.trash.items,
            Err(e) => {
                errors += 1;
                eprintln!("Warning: Failed to empty Recycle Bin: {}", e);
            }
        }
    }
    
    // Clean build artifacts
    if results.build.items > 0 {
        println!("Cleaning build artifacts...");
        for path in &results.build.paths {
            match categories::build::clean(path) {
                Ok(()) => cleaned += 1,
                Err(e) => {
                    errors += 1;
                    eprintln!("Warning: Failed to clean {}: {}", path.display(), e);
                }
            }
        }
    }
    
    if errors > 0 {
        println!("Cleanup complete. {} items cleaned, {} errors encountered.", cleaned, errors);
    } else {
        println!("Cleanup complete. {} items cleaned.", cleaned);
    }
    
    Ok(())
}
