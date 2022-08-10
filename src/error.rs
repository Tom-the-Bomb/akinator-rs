use thiserror::Error as ErrorBase;

use std::time::SystemTimeError;
use serde_json::error::Error as SerdeJsonError;
use reqwest::Error as ReqwestError;
use regex::Error as RegexError;
use std::num::{
    ParseFloatError,
    ParseIntError,
};


#[derive(Debug, ErrorBase)]
pub enum UpdateInfoError {
    #[error("Failed to parse data: {0}")]
    ParseFloatError(#[from] ParseFloatError),

    #[error("Faield to parse data: {0}")]
    ParseIntError(#[from] ParseIntError),
}


#[derive(Debug, ErrorBase)]
pub enum Error {
    #[error("Failed to get current time: {0}")]
    TimeError(#[from] SystemTimeError),

    #[error("RequestError: {0}")]
    RequestError(#[from] ReqwestError),

    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] SerdeJsonError),

    #[error("Failed to update data fields: {0}")]
    UpdateInfoError(#[from] UpdateInfoError),

    #[error("Failed to build regex pattern: {0}")]
    RegexBuildError(#[from] RegexError),

    #[error("Failed to parse API response")]
    ParseResponseError,

    #[error("Failed to find the relevant data")]
    NoDataFound,

    #[error("The akinator servers in that region are currently down")]
    ServersDown,

    #[error("There is a technical error with the akinator servers")]
    TechnicalError,

    #[error("Akinator session timed out")]
    TimeoutError,

    #[error("There are no more available questions")]
    NoMoreQuestions,

    #[error("Failed to connect to akinator servers")]
    ConnectionError,

    #[error("Cannot go back any further, you are already on the first question")]
    CantGoBackAnyFurther,

    #[error("Invalid Answer")]
    InvalidAnswer,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;