use thiserror::Error as ErrorBase;

use std::time::SystemTimeError;
use serde_json::error::Error as SerdeJsonError;
use reqwest::Error as ReqwestError;
use std::num::{
    ParseFloatError,
    ParseIntError,
};


/// returned in the update info methods
#[derive(Debug, ErrorBase)]
pub enum UpdateInfoError {
    /// from propogating [`ParseFloatError`] when parsing json struct fields into [`f32`] fails
    #[error("Failed to parse data: {0}")]
    ParseFloatError(#[from] ParseFloatError),

    /// from propogating [`ParseIntError`] when parsing json struct fields into [`usize`] fails
    #[error("Faield to parse data: {0}")]
    ParseIntError(#[from] ParseIntError),

    /// Deserialized struct is missing a needed field, probably `parameters`
    #[error("Missing an expected json field")]
    MissingData,
}


/// the main Error enum for errors returned from akinator functions
#[derive(Debug, ErrorBase)]
pub enum Error {
    /// from propogating [`SystemTimeError`] when retrieving current timestamp fails when starting the game
    #[error("Failed to get current time: {0}")]
    TimeError(#[from] SystemTimeError),

    /// from propogating [`ReqwestError`] when making HTTP requests fails with reqwests across many akinator functions
    #[error("RequestError: {0}")]
    RequestError(#[from] ReqwestError),

    /// from propogating [`SerdeJsonError`] when deserializing json from [`str`] into a struct fails
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] SerdeJsonError),

    /// conversion from [`UpdateInfoError`]
    #[error("Failed to update data fields: {0}")]
    UpdateInfoError(#[from] UpdateInfoError),

    /// from when searching for information such as the WS url and session info etc.
    #[error("Failed to find the required information needed to start the game")]
    NoDataFound,

    /// from when the akinator servers in the specified region are down
    #[error("The akinator servers in that region are currently down")]
    ServersDown,

    /// from when there is a technical internal error with the akinator servers
    #[error("There is a technical error with the akinator servers")]
    TechnicalError,

    /// from when the akinator session timed out waiting for a response
    #[error("Akinator session timed out waiting for a response")]
    TimeoutError,

    /// from when there are no more available questions the akinator has to offer
    #[error("There are no more available questions")]
    NoMoreQuestions,

    /// from any other form of connection or server error
    #[error("Failed to connect to akinator servers")]
    ConnectionError,

    /// from when calling `back`, fails often when we are already on the first questions so we can't go back any more
    #[error("Cannot go back any further, you are already on the first question")]
    CantGoBackAnyFurther,

    /// Simply an invalid answer to respond to the question when parsing from string
    #[error("Invalid Answer")]
    InvalidAnswer,

    /// from when an invalid or not supported language is passed when parsing from string
    #[error("Invalid Language")]
    InvalidLanguage,
}

/// result typealias with `E`, defaults to [`Error`]
pub type Result<T, E = Error> = std::result::Result<T, E>;