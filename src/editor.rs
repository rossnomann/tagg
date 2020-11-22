use rustyline::{error::ReadlineError, Editor as BaseEditor};
use std::{error::Error as StdError, fmt};

const COMMAND_BACK: &str = ":b";
const COMMAND_QUIT: &str = ":q";

pub struct Editor<S, O> {
    base: BaseEditor<()>,
    state: S,
    output_builder: O,
}

impl<S, O> Editor<S, O>
where
    O: OutputBuilder,
    O::Key: Prompt,
    S: State<O::Key>,
{
    pub fn new(state: S, output_builder: O) -> Self {
        Self {
            base: BaseEditor::new(),
            state,
            output_builder,
        }
    }

    fn read(&mut self, prompt: impl fmt::Display, default_value: DefaultValue) -> Result<ReadlineInput, EditorError> {
        let prompt = format!("[{}] >>> ", prompt);
        let initial = (default_value.left.as_str(), default_value.right.as_str());
        match self.base.readline_with_initial(&prompt, initial) {
            Ok(value) => Ok(match value.trim() {
                COMMAND_BACK => ReadlineInput::Back,
                COMMAND_QUIT => ReadlineInput::Exit,
                value => ReadlineInput::Data(value.to_string()),
            }),
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => Ok(ReadlineInput::Exit),
            Err(err) => Err(EditorError::Readline(err)),
        }
    }

    pub fn run(mut self) -> Result<EditorOutput<O::Output>, EditorError> {
        loop {
            match self.state.get_input() {
                StateInput::Read { key, default_value } => {
                    match self.read(key.get_prompt(), default_value)? {
                        ReadlineInput::Data(value) => {
                            if let Err(err) = self.output_builder.set_value(key, value) {
                                println!("{}", err);
                            } else {
                                self.state.next();
                            }
                        }
                        ReadlineInput::Back => {
                            self.state.prev();
                        }
                        ReadlineInput::Exit => {
                            self.state.interrupt();
                        }
                    };
                }
                StateInput::Interrupted => {
                    return Ok(EditorOutput::Interrupted);
                }
                StateInput::Finished => {
                    return Ok(EditorOutput::Finished(
                        self.output_builder
                            .build()
                            .map_err(|e| EditorError::BuildOutput(Box::new(e)))?,
                    ));
                }
            };
        }
    }
}

#[derive(Debug)]
enum ReadlineInput {
    Data(String),
    Back,
    Exit,
}

#[derive(Debug)]
pub enum EditorOutput<O> {
    Finished(O),
    Interrupted,
}

pub trait State<K: Prompt> {
    fn get_input(&self) -> StateInput<K>;
    fn next(&mut self);
    fn prev(&mut self);
    fn interrupt(&mut self);
    fn finish(&mut self);
}

pub enum StateInput<K: Prompt> {
    Read { key: K, default_value: DefaultValue },
    Interrupted,
    Finished,
}

pub trait Prompt {
    fn get_prompt(&self) -> &str;
}

#[derive(Debug)]
pub struct DefaultValue {
    left: String,
    right: String,
}

impl DefaultValue {
    pub fn new<L, R>(left: L, right: R) -> Self
    where
        L: Into<String>,
        R: Into<String>,
    {
        Self {
            left: left.into(),
            right: right.into(),
        }
    }

    pub fn left<L>(left: L) -> Self
    where
        L: Into<String>,
    {
        Self::new(left, "")
    }

    pub fn right<R>(right: R) -> Self
    where
        R: Into<String>,
    {
        Self::new("", right)
    }
}

impl<T> From<&Option<T>> for DefaultValue
where
    T: fmt::Display,
{
    fn from(value: &Option<T>) -> Self {
        match value {
            Some(value) => Self::left(format!("{}", value)),
            None => Self::right(""),
        }
    }
}

pub trait OutputBuilder {
    type Key;
    type Output;
    type InputError: StdError;
    type OutputError: StdError + 'static;

    fn set_value(&mut self, key: Self::Key, value: String) -> Result<(), Self::InputError>;
    fn build(self) -> Result<Self::Output, Self::OutputError>;
}

#[derive(Debug)]
pub enum EditorError {
    BuildOutput(Box<dyn StdError>),
    Readline(ReadlineError),
}

impl StdError for EditorError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            EditorError::BuildOutput(err) => Some(err.as_ref()),
            EditorError::Readline(err) => Some(err),
        }
    }
}

impl fmt::Display for EditorError {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EditorError::BuildOutput(err) => write!(out, "failed to build output: {}", err),
            EditorError::Readline(err) => write!(out, "{}", err),
        }
    }
}
