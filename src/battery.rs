//! Raw PRG NVRAM saves stored beside the ROM as `<rom filename>.sav`.
use std::{fs, io::{Read, Write}, path::{Path, PathBuf}};

pub fn path_for_rom(rom: &Path) -> PathBuf {
    let mut name = rom.as_os_str().to_os_string();
    name.push(".sav");
    PathBuf::from(name)
}

pub fn load(path: &Path, expected_size: usize) -> Result<Option<Vec<u8>>, String> {
    let file = match fs::File::open(path) {
        Ok(file) => file,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("cannot read battery save {}: {e}", path.display())),
    };
    let mut data = Vec::with_capacity(expected_size);
    file.take(expected_size as u64 + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
    if data.len() != expected_size {
        return Err(format!("battery save {} must contain {expected_size} bytes", path.display()));
    }
    Ok(Some(data))
}

pub fn save(path: &Path, data: &[u8]) -> Result<(), String> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    temp.write_all(data).map_err(|e| e.to_string())?;
    temp.as_file().sync_all().map_err(|e| e.to_string())?;
    temp.persist(path).map_err(|e| format!("cannot replace battery save {}: {e}", path.display()))?;
    Ok(())
}
