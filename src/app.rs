use crate::{
    album::{AlbumEditor, AlbumInput},
    editor::{EditorError, EditorOutput},
    file::{FileOutput, FileOutputError},
    finder::{self, FindError},
    track::{TrackEditor, TrackInput},
};
use std::{
    env,
    error::Error as StdError,
    fmt,
    io::{stdin, stdout, Error as IoError, Write},
    path::PathBuf,
};

fn get_path() -> Result<PathBuf, AppError> {
    let mut args = env::args();
    args.next(); // contains path to executable
    match args.next() {
        Some(path) => {
            let path = PathBuf::from(path);
            if !path.is_dir() {
                Err(AppError::PathNotExists(path))
            } else {
                Ok(path)
            }
        }
        None => env::current_dir().map_err(AppError::GetCurrentDir),
    }
}

pub fn run() -> Result<(), AppError> {
    let root_path = get_path()?;
    let input = finder::find(root_path).map_err(AppError::FindTracks)?;

    let album_input = AlbumInput::from_file_input(&input);
    let album_output = match AlbumEditor::new(album_input).run().map_err(AppError::EditAlbum)? {
        EditorOutput::Finished(output) => output,
        EditorOutput::Interrupted => return Ok(()),
    };
    println!();

    let mut output = Vec::new();
    for item in input {
        println!("{}", item.path.display());
        let track_input = TrackInput::from(&item);
        let track_output = match TrackEditor::new(track_input).run().map_err(AppError::EditTrack)? {
            EditorOutput::Interrupted => return Ok(()),
            EditorOutput::Finished(output) => output,
        };
        output.push(FileOutput::from((item.path, &album_output, track_output)));
        println!();
    }

    loop {
        print!("Continue? [y/n]: ");
        stdout().flush().map_err(AppError::PrintConfirmation)?;
        let mut answer = String::new();
        stdin().read_line(&mut answer).map_err(AppError::ReadConfirmation)?;
        match answer.trim() {
            "y" => {
                for item in output {
                    let path = item.write().map_err(AppError::WriteFile)?;
                    println!("Tags written to {}", path.display());
                }
                break;
            }
            "n" => {
                break;
            }
            _ => println!("Wrong answer!"),
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum AppError {
    EditAlbum(EditorError),
    EditTrack(EditorError),
    FindTracks(FindError),
    GetCurrentDir(IoError),
    PathNotExists(PathBuf),
    PrintConfirmation(IoError),
    ReadConfirmation(IoError),
    WriteFile(FileOutputError),
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use self::AppError::*;
        match self {
            EditAlbum(err) => Some(err),
            EditTrack(err) => Some(err),
            FindTracks(err) => Some(err),
            GetCurrentDir(err) => Some(err),
            PathNotExists(_) => None,
            PrintConfirmation(err) => Some(err),
            ReadConfirmation(err) => Some(err),
            WriteFile(err) => Some(err),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::AppError::*;
        match self {
            EditAlbum(err) => write!(out, "edit album error: {}", err),
            EditTrack(err) => write!(out, "edit track error: {}", err),
            FindTracks(err) => write!(out, "unable to find tracks: {}", err),
            GetCurrentDir(err) => write!(out, "failed to get current directory: {}", err),
            PathNotExists(path) => write!(out, "{} is not a directory", path.display()),
            PrintConfirmation(err) => write!(out, "failed to print confirmation: {}", err),
            ReadConfirmation(err) => write!(out, "failed to read confirmation: {}", err),
            WriteFile(err) => write!(out, "could not write a file: {}", err),
        }
    }
}
