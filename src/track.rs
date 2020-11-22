use crate::{
    editor::{DefaultValue, Editor, EditorError, EditorOutput, OutputBuilder, Prompt, State, StateInput},
    file::FileInput,
};
use std::{error::Error as StdError, fmt, num::ParseIntError};

#[derive(Clone, Debug)]
pub struct TrackInput {
    track_number: Option<u32>,
    disc_number: Option<u32>,
    title: Option<String>,
}

impl From<&FileInput> for TrackInput {
    fn from(input: &FileInput) -> Self {
        Self {
            track_number: input.track_number,
            disc_number: input.disc_number,
            title: input.title.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum TrackKey {
    TrackNumber,
    DiscNumber,
    Title,
}

impl Prompt for TrackKey {
    fn get_prompt(&self) -> &str {
        use self::TrackKey::*;
        match self {
            TrackNumber => "TRACK NUMBER",
            DiscNumber => "DISC NUMBER",
            Title => "TITLE",
        }
    }
}

#[derive(Debug)]
struct TrackOutputBuilder {
    track_input: TrackInput,
}

impl TrackOutputBuilder {
    fn new(track_input: TrackInput) -> Self {
        Self { track_input }
    }
}

impl OutputBuilder for TrackOutputBuilder {
    type Key = TrackKey;
    type Output = TrackOutput;
    type InputError = TrackInputError;
    type OutputError = TrackOutputError;

    fn set_value(&mut self, key: Self::Key, value: String) -> Result<(), Self::InputError> {
        use self::TrackKey::*;
        match key {
            TrackNumber => self.track_input.track_number = Some(value.parse().map_err(TrackInputError::TrackNumber)?),
            DiscNumber => self.track_input.disc_number = Some(value.parse().map_err(TrackInputError::DiscNumber)?),
            Title => self.track_input.title = Some(value),
        }
        Ok(())
    }

    fn build(self) -> Result<Self::Output, Self::OutputError> {
        Ok(TrackOutput {
            track_number: self.track_input.track_number.ok_or(TrackOutputError::TrackNumber)?,
            disc_number: self.track_input.disc_number.ok_or(TrackOutputError::DiscNumber)?,
            title: self.track_input.title.ok_or(TrackOutputError::Title)?,
        })
    }
}

#[derive(Debug)]
pub struct TrackOutput {
    pub track_number: u32,
    pub disc_number: u32,
    pub title: String,
}

#[derive(Debug)]
enum TrackInputError {
    TrackNumber(ParseIntError),
    DiscNumber(ParseIntError),
}

impl StdError for TrackInputError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use self::TrackInputError::*;
        match self {
            TrackNumber(err) => Some(err),
            DiscNumber(err) => Some(err),
        }
    }
}

impl fmt::Display for TrackInputError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::TrackInputError::*;
        match self {
            TrackNumber(err) => write!(out, "invalid track number: {}", err),
            DiscNumber(err) => write!(out, "invalid disc number: {}", err),
        }
    }
}

#[derive(Debug)]
enum TrackOutputError {
    TrackNumber,
    DiscNumber,
    Title,
}

impl StdError for TrackOutputError {}

impl fmt::Display for TrackOutputError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        use self::TrackOutputError::*;
        write!(
            out,
            "{} is required",
            match self {
                TrackNumber => "track number",
                DiscNumber => "disc number",
                Title => "title",
            }
        )
    }
}

struct TrackState {
    track_input: TrackInput,
    kind: TrackStateKind,
}

enum TrackStateKind {
    TrackNumber,
    DiscNumber,
    Title,
    Interrupted,
    Finished,
}

impl TrackState {
    fn new(track_input: TrackInput) -> Self {
        Self {
            track_input,
            kind: TrackStateKind::TrackNumber,
        }
    }
}

impl State<TrackKey> for TrackState {
    fn get_input(&self) -> StateInput<TrackKey> {
        use self::TrackStateKind::*;
        match self.kind {
            TrackNumber => StateInput::Read {
                key: TrackKey::TrackNumber,
                default_value: DefaultValue::from(&self.track_input.track_number),
            },
            DiscNumber => StateInput::Read {
                key: TrackKey::DiscNumber,
                default_value: DefaultValue::from(&self.track_input.disc_number),
            },
            Title => StateInput::Read {
                key: TrackKey::Title,
                default_value: DefaultValue::from(&self.track_input.title),
            },
            Interrupted => StateInput::Interrupted,
            Finished => StateInput::Finished,
        }
    }

    fn next(&mut self) {
        use self::TrackStateKind::*;
        self.kind = match self.kind {
            TrackNumber => DiscNumber,
            DiscNumber => Title,
            Title => Finished,
            Interrupted => Interrupted,
            Finished => Finished,
        }
    }

    fn prev(&mut self) {
        use self::TrackStateKind::*;
        self.kind = match self.kind {
            TrackNumber => TrackNumber,
            DiscNumber => TrackNumber,
            Title => DiscNumber,
            Interrupted => TrackNumber,
            Finished => Title,
        }
    }

    fn interrupt(&mut self) {
        self.kind = TrackStateKind::Interrupted;
    }

    fn finish(&mut self) {
        self.kind = TrackStateKind::Finished;
    }
}

pub struct TrackEditor {
    inner: Editor<TrackState, TrackOutputBuilder>,
}

impl TrackEditor {
    pub fn new(track_input: TrackInput) -> Self {
        Self {
            inner: Editor::new(
                TrackState::new(track_input.clone()),
                TrackOutputBuilder::new(track_input),
            ),
        }
    }

    pub fn run(self) -> Result<EditorOutput<TrackOutput>, EditorError> {
        self.inner.run()
    }
}
