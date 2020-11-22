use crate::{
    counter::Counter,
    editor::{DefaultValue, Editor, EditorError, EditorOutput, OutputBuilder, Prompt, State, StateInput},
    file::FileInput,
};
use std::{error::Error as StdError, fmt, num::ParseIntError};

#[derive(Clone, Debug, Default)]
pub struct AlbumInput {
    pub artist: Option<String>,
    pub album_artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<i32>,
    pub total_tracks: Option<u32>,
    pub total_discs: Option<u32>,
}

impl AlbumInput {
    pub fn from_file_input(items: &[FileInput]) -> Self {
        let mut counter = Counter::default();
        for item in items {
            if let Some(ref artist) = item.artist {
                counter.insert(AlbumKey::Artist, artist.clone());
            }
            if let Some(ref album_artist) = item.album_artist {
                counter.insert(AlbumKey::AlbumArtist, album_artist.clone());
            }
            if let Some(ref album) = item.album {
                counter.insert(AlbumKey::Album, album.clone());
            }
            if let Some(year) = item.year {
                counter.insert(AlbumKey::Year, format!("{}", year));
            }
            if let Some(total_tracks) = item.total_tracks {
                counter.insert(AlbumKey::TotalTracks, format!("{}", total_tracks));
            }
            if let Some(total_discs) = item.total_discs {
                counter.insert(AlbumKey::TotalDiscs, format!("{}", total_discs));
            }
        }
        Self {
            artist: counter.most_common(AlbumKey::Artist),
            album_artist: counter.most_common(AlbumKey::AlbumArtist),
            album: counter.most_common(AlbumKey::Album),
            year: counter.most_common(AlbumKey::Year).and_then(|x| x.parse().ok()),
            total_tracks: counter.most_common(AlbumKey::TotalTracks).and_then(|x| x.parse().ok()),
            total_discs: counter.most_common(AlbumKey::TotalDiscs).and_then(|x| x.parse().ok()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum AlbumKey {
    Artist,
    AlbumArtist,
    Album,
    Year,
    TotalTracks,
    TotalDiscs,
}

impl Prompt for AlbumKey {
    fn get_prompt(&self) -> &str {
        use self::AlbumKey::*;
        match self {
            Artist => "ARTIST",
            AlbumArtist => "ALBUM ARTIST",
            Album => "ALBUM",
            Year => "YEAR",
            TotalTracks => "TOTAL TRACKS",
            TotalDiscs => "TOTAL DISCS",
        }
    }
}

#[derive(Debug)]
struct AlbumOutputBuilder {
    album_input: AlbumInput,
}

impl AlbumOutputBuilder {
    fn new(album_input: AlbumInput) -> Self {
        Self { album_input }
    }
}

impl OutputBuilder for AlbumOutputBuilder {
    type Key = AlbumKey;
    type Output = AlbumOutput;
    type InputError = AlbumInputError;
    type OutputError = AlbumOutputError;

    fn set_value(&mut self, key: Self::Key, value: String) -> Result<(), Self::InputError> {
        use self::AlbumKey::*;
        match key {
            Artist => self.album_input.artist = Some(value),
            AlbumArtist => self.album_input.album_artist = Some(value),
            Album => self.album_input.album = Some(value),
            Year => self.album_input.year = Some(value.parse().map_err(AlbumInputError::Year)?),
            TotalTracks => self.album_input.total_tracks = Some(value.parse().map_err(AlbumInputError::TotalTracks)?),
            TotalDiscs => self.album_input.total_discs = Some(value.parse().map_err(AlbumInputError::TotalDiscs)?),
        }
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::OutputError> {
        Ok(AlbumOutput {
            artist: self.album_input.artist.ok_or(AlbumOutputError::Artist)?,
            album_artist: self.album_input.album_artist.ok_or(AlbumOutputError::AlbumArtist)?,
            album: self.album_input.album.ok_or(AlbumOutputError::Album)?,
            year: self.album_input.year.ok_or(AlbumOutputError::Year)?,
            total_tracks: self.album_input.total_tracks.ok_or(AlbumOutputError::TotalTracks)?,
            total_discs: self.album_input.total_discs.ok_or(AlbumOutputError::TotalDiscs)?,
        })
    }
}

#[derive(Debug)]
pub struct AlbumOutput {
    pub artist: String,
    pub album_artist: String,
    pub album: String,
    pub year: i32,
    pub total_tracks: u32,
    pub total_discs: u32,
}

#[derive(Debug)]
enum AlbumInputError {
    Year(ParseIntError),
    TotalTracks(ParseIntError),
    TotalDiscs(ParseIntError),
}

impl StdError for AlbumInputError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use self::AlbumInputError::*;
        match self {
            Year(err) => Some(err),
            TotalTracks(err) => Some(err),
            TotalDiscs(err) => Some(err),
        }
    }
}

impl fmt::Display for AlbumInputError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::AlbumInputError::*;
        match self {
            Year(err) => write!(out, "invalid year: {}", err),
            TotalTracks(err) => write!(out, "invalid number of tracks: {}", err),
            TotalDiscs(err) => write!(out, "invalid number of discs: {}", err),
        }
    }
}

#[derive(Debug)]
enum AlbumOutputError {
    Artist,
    AlbumArtist,
    Album,
    Year,
    TotalTracks,
    TotalDiscs,
}

impl StdError for AlbumOutputError {}

impl fmt::Display for AlbumOutputError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::AlbumOutputError::*;
        write!(
            out,
            "{} is required",
            match self {
                Artist => "artist",
                AlbumArtist => "album artist",
                Album => "album",
                Year => "year",
                TotalTracks => "number of tracks",
                TotalDiscs => "number of discs",
            }
        )
    }
}

struct AlbumState {
    album_input: AlbumInput,
    kind: AlbumStateKind,
}

enum AlbumStateKind {
    Artist,
    AlbumArtist,
    Album,
    Year,
    TotalTracks,
    TotalDiscs,
    Interrupted,
    Finished,
}

impl AlbumState {
    fn new(album_input: AlbumInput) -> Self {
        Self {
            album_input,
            kind: AlbumStateKind::Artist,
        }
    }
}

impl State<AlbumKey> for AlbumState {
    fn get_input(&self) -> StateInput<AlbumKey> {
        use self::AlbumStateKind::*;
        match self.kind {
            Artist => StateInput::Read {
                key: AlbumKey::Artist,
                default_value: DefaultValue::from(&self.album_input.artist),
            },
            AlbumArtist => StateInput::Read {
                key: AlbumKey::AlbumArtist,
                default_value: DefaultValue::from(&self.album_input.album_artist),
            },
            Album => StateInput::Read {
                key: AlbumKey::Album,
                default_value: DefaultValue::from(&self.album_input.album),
            },
            Year => StateInput::Read {
                key: AlbumKey::Year,
                default_value: DefaultValue::from(&self.album_input.year),
            },
            TotalTracks => StateInput::Read {
                key: AlbumKey::TotalTracks,
                default_value: DefaultValue::from(&self.album_input.total_tracks),
            },
            TotalDiscs => StateInput::Read {
                key: AlbumKey::TotalDiscs,
                default_value: DefaultValue::from(&self.album_input.total_discs),
            },
            Interrupted => StateInput::Interrupted,
            Finished => StateInput::Finished,
        }
    }

    fn next(&mut self) {
        use self::AlbumStateKind::*;
        self.kind = match self.kind {
            Artist => AlbumArtist,
            AlbumArtist => Album,
            Album => Year,
            Year => TotalTracks,
            TotalTracks => TotalDiscs,
            TotalDiscs => Finished,
            Interrupted => Interrupted,
            Finished => Finished,
        };
    }

    fn prev(&mut self) {
        use self::AlbumStateKind::*;
        self.kind = match self.kind {
            Artist => Artist,
            AlbumArtist => Artist,
            Album => AlbumArtist,
            Year => Album,
            TotalTracks => Year,
            TotalDiscs => TotalTracks,
            Interrupted => Artist,
            Finished => TotalDiscs,
        };
    }

    fn interrupt(&mut self) {
        self.kind = AlbumStateKind::Interrupted;
    }

    fn finish(&mut self) {
        self.kind = AlbumStateKind::Finished;
    }
}

pub struct AlbumEditor {
    inner: Editor<AlbumState, AlbumOutputBuilder>,
}

impl AlbumEditor {
    pub fn new(album_input: AlbumInput) -> Self {
        Self {
            inner: Editor::new(
                AlbumState::new(album_input.clone()),
                AlbumOutputBuilder::new(album_input),
            ),
        }
    }

    pub fn run(self) -> Result<EditorOutput<AlbumOutput>, EditorError> {
        self.inner.run()
    }
}
