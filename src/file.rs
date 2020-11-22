use crate::{album::AlbumOutput, track::TrackOutput};
use ape::Error as ApeError;
use id3::{
    v1::Tag as Id3V1Tag, Content as Id3FrameContent, Error as Id3Error, Frame as Id3Frame, Tag as Id3V2Tag,
    Version as Id3Version,
};
use std::{
    error::Error as StdError,
    fmt,
    fs::{rename, OpenOptions},
    io::Error as IoError,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct FileInput {
    pub path: PathBuf,
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub title: Option<String>,
    pub track_number: Option<u32>,
    pub total_tracks: Option<u32>,
    pub disc_number: Option<u32>,
    pub total_discs: Option<u32>,
}

impl FileInput {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Id3Error> {
        let path = path.as_ref();
        let tag = Id3V2Tag::read_from_path(path)?;
        Ok(Self {
            path: path.to_owned(),
            artist: tag.artist().map(ToOwned::to_owned),
            album_artist: tag.album_artist().map(ToOwned::to_owned),
            album: tag.album().map(ToOwned::to_owned),
            year: tag.date_recorded().map(|x| x.year).or_else(|| tag.year()),
            title: tag.title().map(ToOwned::to_owned),
            track_number: tag.track(),
            total_tracks: tag.total_tracks(),
            disc_number: tag.disc(),
            total_discs: tag.total_discs(),
        })
    }
}

#[derive(Debug)]
pub struct FileOutput {
    path: PathBuf,
    artist: String,
    album_artist: String,
    album: String,
    year: i32,
    title: String,
    track_number: u32,
    total_tracks: u32,
    disc_number: u32,
    total_discs: u32,
}

impl From<(PathBuf, &AlbumOutput, TrackOutput)> for FileOutput {
    fn from((path, album_output, track_output): (PathBuf, &AlbumOutput, TrackOutput)) -> Self {
        Self {
            path,
            artist: album_output.artist.clone(),
            album_artist: album_output.album_artist.clone(),
            album: album_output.album.clone(),
            year: album_output.year,
            title: track_output.title,
            track_number: track_output.track_number,
            total_tracks: album_output.total_tracks,
            disc_number: track_output.disc_number,
            total_discs: album_output.total_discs,
        }
    }
}

impl FileOutput {
    pub fn write(self) -> Result<PathBuf, FileOutputError> {
        ape::remove(&self.path).map_err(FileOutputError::RemoveApe)?;
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)
            .map_err(FileOutputError::OpenFile)?;
        Id3V1Tag::remove(&mut file).map_err(FileOutputError::RemoveId3V1)?;
        Id3V2Tag::remove_from(&mut file).map_err(FileOutputError::RemoveId3V2)?;
        let mut tag = Id3V2Tag::new();
        tag.add_frame(Id3Frame::with_content("TPE1", Id3FrameContent::Text(self.artist)));
        tag.add_frame(Id3Frame::with_content("TPE2", Id3FrameContent::Text(self.album_artist)));
        tag.add_frame(Id3Frame::with_content("TALB", Id3FrameContent::Text(self.album)));
        tag.add_frame(Id3Frame::with_content(
            "TDRC",
            Id3FrameContent::Text(self.year.to_string()),
        ));
        tag.add_frame(Id3Frame::with_content(
            "TIT2",
            Id3FrameContent::Text(self.title.clone()),
        ));
        tag.add_frame(Id3Frame::with_content(
            "TRCK",
            Id3FrameContent::Text(format!("{:02}/{:02}", self.track_number, self.total_tracks)),
        ));
        tag.add_frame(Id3Frame::with_content(
            "TPOS",
            Id3FrameContent::Text(format!("{:02}/{:02}", self.disc_number, self.total_discs)),
        ));
        tag.write_to(file, Id3Version::Id3v24)
            .map_err(FileOutputError::WriteId3V2)?;

        let number = if self.total_discs > 1 {
            format!("{:02}-{:02}", self.disc_number, self.track_number)
        } else {
            format!("{:02}", self.track_number)
        };
        let filename = format!("{} - {}.mp3", number, self.title);
        let new_path = self.path.with_file_name(filename);
        rename(&self.path, &new_path).map_err(FileOutputError::RenameFile)?;

        Ok(new_path)
    }
}

#[derive(Debug)]
pub enum FileOutputError {
    OpenFile(IoError),
    RemoveApe(ApeError),
    RemoveId3V1(Id3Error),
    RemoveId3V2(Id3Error),
    RenameFile(IoError),
    WriteId3V2(Id3Error),
}

impl StdError for FileOutputError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use self::FileOutputError::*;
        match self {
            OpenFile(err) => Some(err),
            RemoveApe(err) => Some(err),
            RemoveId3V1(err) => Some(err),
            RemoveId3V2(err) => Some(err),
            RenameFile(err) => Some(err),
            WriteId3V2(err) => Some(err),
        }
    }
}

impl fmt::Display for FileOutputError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::FileOutputError::*;
        match self {
            OpenFile(err) => write!(out, "failed to open file: {}", err),
            RemoveApe(err) => write!(out, "failed to remove APE tag: {}", err),
            RemoveId3V1(err) => write!(out, "failed to remove ID3V1 tag: {}", err),
            RemoveId3V2(err) => write!(out, "failed to remove ID3V2 tag: {}", err),
            RenameFile(err) => write!(out, "failed to rename file: {}", err),
            WriteId3V2(err) => write!(out, "failed to write ID3V2 tag: {}", err),
        }
    }
}
