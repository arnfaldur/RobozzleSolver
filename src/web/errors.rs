use thirtyfour::fantoccini::error::CmdError;
use thirtyfour::prelude::*;

#[derive(Debug)]
pub enum SolverError {
    Fantoccini(CmdError),
    IOError(std::io::Error),
    Error(String),
    WebDriver(WebDriverError),
    Serde(serde_json::Error),
    NoPuzzleForId,
}

impl From<serde_json::Error> for SolverError {
    fn from(value: serde_json::Error) -> Self {
        SolverError::Serde(value)
    }
}

impl From<CmdError> for SolverError {
    fn from(err: CmdError) -> Self {
        SolverError::Fantoccini(err)
    }
}

impl From<SolverError> for CmdError {
    fn from(err: SolverError) -> Self {
        match err {
            SolverError::Fantoccini(e) => e,
            _ => CmdError::InvalidArgument(
                "Couldn't convert from Solver to Cmd error".to_string(),
                "sorry".to_string(),
            ),
        }
    }
}

impl From<SolverError> for WebDriverError {
    fn from(value: SolverError) -> Self {
        match value {
            SolverError::WebDriver(err) => err,
            _ => {
                WebDriverError::CustomError("Couldn't convert from Solver to Cmd error".to_string())
            }
        }
    }
}

impl From<std::io::Error> for SolverError {
    fn from(err: std::io::Error) -> Self {
        SolverError::IOError(err)
    }
}

impl From<WebDriverError> for SolverError {
    fn from(value: WebDriverError) -> Self {
        SolverError::WebDriver(value)
    }
}
