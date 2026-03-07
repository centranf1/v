use std::fs::OpenOptions;
use std::io::{self, Write};
use std::path::Path;

/// Perform an atomic write: write data to a temporary file and then rename.
/// Returns `Ok(())` on success, or an io::Error on failure.
pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    // Implementation will go here; tests drive development.
    let tmp_path = path.with_extension("tmp");
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_path)?;
    file.write_all(data)?;
    file.flush()?;
    // Ensure data is on disk
    file.sync_all()?;
    std::fs::rename(&tmp_path, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    #[test]
    fn test_atomic_write_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let data = b"hello world";
        atomic_write(&file_path, data).unwrap();
        let mut contents = Vec::new();
        File::open(&file_path)
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, data);
    }

    #[test]
    fn test_atomic_write_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test2.txt");
        atomic_write(&file_path, b"first").unwrap();
        atomic_write(&file_path, b"second").unwrap();
        let mut contents = Vec::new();
        File::open(&file_path)
            .unwrap()
            .read_to_end(&mut contents)
            .unwrap();
        assert_eq!(contents, b"second");
    }

    #[test]
    fn test_atomic_write_checksum() {
        // later, checksum requirement
    }
}
