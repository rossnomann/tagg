use crate::file::FileInput;
use std::{
    error::Error,
    fmt, fs,
    io::Error as IoError,
    path::{Path, PathBuf},
};

const EXT_MP3: &str = "mp3";

pub fn find(path: impl AsRef<Path>) -> Result<Vec<FileInput>, FindError> {
    let path = path.as_ref();
    let mut result = Vec::new();
    for entry in fs::read_dir(&path).map_err(|err| FindError::ReadDir(path.to_owned(), err))? {
        let entry = entry.map_err(FindError::ReadEntry)?;
        let entry_path = entry.path();
        if !entry_path.is_file() {
            continue;
        }
        if !entry_path
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.to_lowercase() == EXT_MP3)
            .unwrap_or(false)
        {
            continue;
        }
        result.push(FileInput::from_path(&entry_path))
    }
    if result.is_empty() {
        Err(FindError::NoTracks)
    } else {
        Ok(result)
    }
}

#[derive(Debug)]
pub enum FindError {
    NoTracks,
    ReadDir(PathBuf, IoError),
    ReadEntry(IoError),
}

impl fmt::Display for FindError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::FindError::*;
        match self {
            NoTracks => write!(out, "no tracks found"),
            ReadDir(path, err) => write!(out, "failed to read a directory {}: {}", path.display(), err),
            ReadEntry(err) => write!(out, "failed to read an entry: {}", err),
        }
    }
}

impl Error for FindError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        use self::FindError::*;
        Some(match self {
            NoTracks => return None,
            ReadDir(_, err) => err,
            ReadEntry(err) => err,
        })
    }
}
