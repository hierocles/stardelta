use std::path::Path;
use std::io::Cursor;
use ba2::fo4::{Archive, ArchiveKey, FileWriteOptions};
use ba2::prelude::*;

pub struct Ba2Path {
    pub archive_path: String,
    pub file_path: String,
}

impl Ba2Path {
    pub fn from_string(path: &str) -> Option<Ba2Path> {
        // Format expected: "archive.ba2//internal/path/to/file.swf"
        let parts: Vec<&str> = path.split("//").collect();
        if parts.len() == 2 {
            Some(Ba2Path {
                archive_path: parts[0].to_string(),
                file_path: parts[1].to_string(),
            })
        } else {
            None
        }
    }
}

pub fn extract_file_from_ba2(ba2_path: &Ba2Path) -> Result<Vec<u8>, String> {
    let archive_path = Path::new(&ba2_path.archive_path);

    // Open and read the archive
    let (archive, meta) = Archive::read(archive_path)
        .map_err(|e| format!("Failed to open BA2 archive: {}", e))?;

    // Create the archive key from the file path
    let key: ArchiveKey = ba2_path.file_path.as_bytes().into();

    // Get the file from the archive
    let file = archive.get(&key)
        .ok_or_else(|| format!("File '{}' not found in archive", ba2_path.file_path))?;

    // Create a buffer to hold the file data
    let mut buffer = Vec::new();
    {
        let mut cursor = Cursor::new(&mut buffer);

        // Extract the file using the metadata options
        let options: FileWriteOptions = meta.into();
        file.write(&mut cursor, &options)
            .map_err(|e| format!("Failed to extract file from BA2: {}", e))?;
    }

    Ok(buffer)
}

pub fn is_ba2_path(path: &str) -> bool {
    Ba2Path::from_string(path).is_some()
}
