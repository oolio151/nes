use std::{io::{self, IsTerminal, Write}, path::{Path, PathBuf}};
use dialoguer::Select;

/// Accept a ROM file directly, or browse the .nes files immediately inside a directory.
pub fn prompt_for_rom_path() -> io::Result<Option<String>> {
    loop {
        print!("Enter path to ROM file or directory: ");
        io::stdout().flush()?;
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 { return Ok(None); }
        let input = input.trim();
        // Accept paths pasted with matching quotes, including paths containing spaces.
        let input = input.strip_prefix('"').and_then(|s| s.strip_suffix('"'))
            .or_else(|| input.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
            .unwrap_or(input);
        if input.is_empty() {
            println!("Path cannot be empty, try again.");
            continue;
        }
        let path = Path::new(input);
        let selected = if path.is_dir() {
            match select_rom(path) {
                Ok(Some(path)) => path,
                Ok(None) => continue,
                Err(error) => {
                    println!("Unable to select ROM: {error}");
                    continue;
                }
            }
        } else if path.is_file() {
            path.to_path_buf()
        } else {
            println!("No file or directory found at '{}', try again.", path.display());
            continue;
        };
        match selected.into_os_string().into_string() {
            Ok(path) => return Ok(Some(path)),
            Err(_) => println!("This ROM path contains unsupported characters; rename it and try again."),
        }
    }
}

fn select_rom(directory: &Path) -> io::Result<Option<PathBuf>> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("nes"))
        {
            files.push(path);
        }
    }
    files.sort_by(|a, b| {
        a.file_name().unwrap().to_string_lossy().to_lowercase()
            .cmp(&b.file_name().unwrap().to_string_lossy().to_lowercase())
            .then_with(|| a.cmp(b))
    });
    if files.is_empty() {
        println!("No .nes files found in '{}'.", directory.display());
        return Ok(None);
    }
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return Err(io::Error::other("directory selection needs an interactive terminal; enter a ROM file path instead"));
    }
    // Keep control characters in filenames from affecting the terminal display.
    let names: Vec<String> = files.iter().map(|path| {
        path.file_name().unwrap().to_string_lossy().chars()
            .map(|c| if c.is_control() { '\u{fffd}' } else { c }).collect()
    }).collect();
    let selected = Select::new()
        .with_prompt("Choose a ROM (Up/Down, Enter to select; Esc to go back)")
        .items(&names)
        .default(0)
        .max_length(12)
        .interact_opt()
        .map_err(io::Error::other)?;
    Ok(selected.map(|index| files.swap_remove(index)))
}
