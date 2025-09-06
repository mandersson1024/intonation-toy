use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleOutput {
    Info(String),
    Success(String),
    Warning(String),
    Error(String),
    Echo(String),
    Empty,
}

impl ConsoleOutput {
    pub fn info(message: impl Into<String>) -> Self {
        Self::Info(message.into())
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::Success(message.into())
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::Warning(message.into())
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::Error(message.into())
    }

    pub fn echo(message: impl Into<String>) -> Self {
        Self::Echo(message.into())
    }

    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn output_type(&self) -> &'static str {
        match self {
            Self::Info(_) => "info",
            Self::Success(_) => "success",
            Self::Warning(_) => "warning",
            Self::Error(_) => "error",
            Self::Echo(_) => "command",
            Self::Empty => "empty",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            Self::Info(msg) | Self::Success(msg) | Self::Warning(msg) 
            | Self::Error(msg) | Self::Echo(msg) => msg,
            Self::Empty => "",
        }
    }
}

impl fmt::Display for ConsoleOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info(msg) => write!(f, "[INFO] {}", msg),
            Self::Success(msg) => write!(f, "[SUCCESS] {}", msg),
            Self::Warning(msg) => write!(f, "[WARNING] {}", msg),
            Self::Error(msg) => write!(f, "[ERROR] {}", msg),
            Self::Echo(msg) => write!(f, "> {}", msg),
            Self::Empty => write!(f, ""),
        }
    }
}

const MAX_OUTPUT_ENTRIES: usize = 1000;

#[derive(Debug, Clone, Default)]
pub struct ConsoleOutputManager {
    entries: Vec<ConsoleOutput>,
}

impl ConsoleOutputManager {
    pub fn add_output(&mut self, output: ConsoleOutput) {
        self.entries.insert(0, output);
        if self.entries.len() > MAX_OUTPUT_ENTRIES {
            self.entries.truncate(MAX_OUTPUT_ENTRIES);
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn entries(&self) -> &[ConsoleOutput] {
        &self.entries
    }
}

