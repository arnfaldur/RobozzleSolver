use std::fs::File;
use std::io::{prelude::*, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thirtyfour::extensions::addons::firefox::FirefoxTools;
use thirtyfour::extensions::query::conditions;
use thirtyfour::fantoccini::error::CmdError;
use thirtyfour::prelude::*;
use tokio::runtime::Runtime;
use webdriver::common::LocatorStrategy::CSSSelector;

use crate::constants::*;
use crate::game::{instructions::*, make_puzzle, Direction, Puzzle, Source, Tile};
use crate::solver::backtrack::backtrack;
use tokio::io::AsyncRead;

#[derive(Debug)]
pub enum SolverError {
    Fantoccini(CmdError),
    IOError(std::io::Error),
    Error(String),
    WebDriver(WebDriverError),
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
