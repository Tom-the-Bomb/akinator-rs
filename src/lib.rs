//! A simple wrapper crate around the Akinator API

use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use reqwest::{
    Client,
    header::{
        HeaderMap, HeaderName, HeaderValue, USER_AGENT,
    },
};

use crate::{
    enums::{Theme, Answer, Language},
    error::{
        Result,
        Error,
        UpdateInfoError,
    },
};

pub mod models;
pub mod error;
pub mod enums;


lazy_static! {
    static ref HEADERS: HeaderMap<HeaderValue> = {
        let mut headers = HeaderMap::new();

        headers.insert(
            USER_AGENT,
            HeaderValue::from_static(
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) snap Chromium/81.0.4044.92 Chrome/81.0.4044.92 Safari/537.36"
            ),
        );
        headers.insert(
            HeaderName::from_static("x-requested-with"),
            HeaderValue::from_static(
                "XMLHttpRequest"
            ),
        );
        headers
    };
}

/// simple macro for retrieving an `Option` field's value
/// to avoid repetition as this is frequently used
macro_rules! get_field {
    ( $field:expr ) => {
        $field.as_ref()
            .ok_or(Error::NoDataFound)?
            .to_string()
    }
}


/// Represents an akinator game
#[derive(Debug, Clone)]
pub struct Akinator {
    /// The language for the akinator session
    pub language: Language,
    /// The theme for the akinator session
    ///
    /// One of 'Characters', 'Animals', or 'Objects'
    pub theme: Theme,
    /// indicates whether or not to filter out NSFW questions and content
    pub child_mode: bool,

    /// The reqwest client used for this akinator session
    http_client: Client,
    /// The POSIX timestamp the game session was started
    /// used for keeping track of sessions
    timestamp: u64,
    /// the base URI to use when making requests
    /// usually: https://{language}.akinator.com/
    uri: String,
    /// The unique identifier for the akinator session
    uid: Option<String>,
    /// the websocket url (server) used for the game
    ws_url: Option<String>,
    /// a (0 - 100) number representing the game's session
    session: Option<usize>,
    /// An IP address encoded in Base64, for authentication purposes
    frontaddr: Option<String>,
    /// A 9 - 10ish digit number that represents the game's signature
    signature: Option<usize>,
    question_filter: Option<String>,

    /// returns the current question to answer
    pub current_question: Option<String>,
    /// returns the progress of the akinator
    /// a float out of 100.0
    pub progression: f32,
    /// returns the a counter of questions asked and answered
    /// starts at 0
    pub step: usize,

    /// returns the akinator's best guess
    ///
    /// Only will be set when [`Self::win`] has been called
    pub first_guess: Option<models::Guess>,
    /// a vec containing all the possible guesses by the akinator
    ///
    /// Only will be set when [`Self::win`] has been called
    pub guesses: Vec<models::Guess>,
}

impl Akinator {
    /// Creates a new [`Akinator`] instance
    /// with fields filled with default values
    ///
    /// # Errors
    /// If failed to create HTTP [`reqwest`] client
    pub fn new() -> Result<Self> {
        Ok(Self {
            language: Language::default(),
            theme: Theme::default(),
            child_mode: false,

            http_client: Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?,
            timestamp: 0,
            uri: "https://en.akinator.com".to_string(),
            uid: None,
            ws_url: None,
            session: None,
            frontaddr: None,
            signature: None,
            question_filter: None,

            current_question: None,
            progression: 0.0,
            step: 0,

            first_guess: None,
            guesses: Vec::new(),
        })
    }

    /// builder method to set the [`Self.theme`] for the akinator game
    #[must_use]
    pub const fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// builder method to set the [`Self.language`] for the akinator game
    #[must_use]
    pub const fn with_language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// builder function to turn on [`Self.child_mode`]
    #[must_use]
    pub const fn with_child_mode(mut self) -> Self {
        self.child_mode = true;
        self
    }

    /// Internal method to handle an error response from the akinator API
    /// and return an appropriate Err value
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    fn handle_error_response(completion: String) -> Error {
        match completion.to_uppercase().as_str() {
            "KO - SERVER DOWN" => Error::ServersDown,
            "KO - TECHNICAL ERROR" => Error::TechnicalError,
            "KO - TIMEOUT" => Error::TimeoutError,
            "KO - ELEM LIST IS EMPTY" | "WARN - NO QUESTION" => Error::NoMoreQuestions,
            _ => Error::ConnectionError,
        }
    }

    /// internal method used to parse and find the [`Self.ws_url`] for this game
    async fn find_server(&self) -> Result<String> {
        lazy_static! {
            static ref DATA_REGEX: Regex = RegexBuilder::new(
                r#"\[\{"translated_theme_name":".*","urlWs":"https:\\/\\/srv[0-9]+\.akinator\.com:[0-9]+\\/ws","subject_id":"[0-9]+"\}\]"#
            )
                .case_insensitive(true)
                .multi_line(true)
                .build()
                .unwrap();
        }

        let html = self.http_client.get(&self.uri)
            .send()
            .await?
            .text()
            .await?;

        let id = (self.theme as usize)
            .to_string();

        if let Some(mat) = DATA_REGEX.find(html.as_str()) {
            let json: Vec<models::ServerData> =
                serde_json::from_str(mat.as_str())?;

            let mat = json
                .into_iter()
                .find(|entry| entry.subject_id == id)
                .ok_or(Error::NoDataFound)?;

            Ok(mat.url_ws)
        } else {
            Err(Error::NoDataFound)
        }
    }

    /// internal method used to parse and find the session uid and frontaddr for the akinator session
    ///
    /// Done by parsing the javascript of the site, extracting variable values
    async fn find_session_info(&self) -> Result<(String, String)> {
        lazy_static! {
            static ref VARS_REGEX: Regex =
                RegexBuilder::new(r"var uid_ext_session = '(.*)';\n.*var frontaddr = '(.*)';")
                    .case_insensitive(true)
                    .multi_line(true)
                    .build()
                    .unwrap();
        }

        let html = self.http_client
            .get("https://en.akinator.com/game")
            .send()
            .await?
            .text()
            .await?;

        if let Some(mat) = VARS_REGEX.captures(html.as_str()) {
            let result = (
                mat.get(1).ok_or(Error::NoDataFound)?
                    .as_str().to_string(),
                mat.get(2).ok_or(Error::NoDataFound)?
                    .as_str().to_string(),
            );

            Ok(result)
        } else {
            Err(Error::NoDataFound)
        }
    }

    /// internal method used to parse the response returned from the API
    ///
    /// strips the function call wrapped around the json, returning the json string
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    fn parse_response(html: String) -> String {
        lazy_static! {
            static ref RESPONSE_REGEX: Regex =
                RegexBuilder::new(r"^jQuery\d+_\d+\(")
                    .case_insensitive(true)
                    .multi_line(true)
                    .build()
                    .unwrap();
        }

        RESPONSE_REGEX
            .replace(html.as_str(), "")
            .strip_suffix(')')
            .unwrap_or(html.as_str())
            .to_string()
    }

    /// updates the [`Akinator`] fields after each response
    fn update_move_info(&mut self, json: models::MoveJson) -> Result<(), UpdateInfoError> {
        let params = json.parameters
            .ok_or(UpdateInfoError::MissingData)?;

        self.current_question = Some(
            params.question
        );

        self.progression = params.progression
            .parse::<f32>()?;

        self.step = params.step
            .parse::<usize>()?;

        Ok(())
    }

    /// similar to [`Self::update_move_info`], but only called once when [`Self::start`] is called
    fn update_start_info(&mut self, json: &models::StartJson) -> Result<(), UpdateInfoError> {
        let ident = &json.parameters
            .as_ref()
            .ok_or(UpdateInfoError::MissingData)?
            .identification;

        let step_info = &json.parameters
            .as_ref()
            .ok_or(UpdateInfoError::MissingData)?
            .step_information;

        self.session = Some(
            ident.session
                .parse::<usize>()?
        );

        self.signature = Some(
            ident.signature
                .parse::<usize>()?
        );

        self.current_question = Some(
            step_info.question.clone()
        );

        self.progression = step_info.progression
            .parse::<f32>()?;

        self.step = step_info.step
            .parse::<usize>()?;

        Ok(())
    }

    /// Starts the akinator game and returns the first question
    ///
    /// # Errors
    ///
    /// see [errors](https://docs.rs/akinator-rs/latest/akinator_rs/error/enum.Error.html) docs for more info
    pub async fn start(&mut self) -> Result<Option<String>> {
        self.uri = format!("https://{}.akinator.com", self.language);
        self.ws_url = Some(self.find_server().await?);

        let (uid, frontaddr) = self.find_session_info().await?;
        self.uid = Some(uid);
        self.frontaddr = Some(frontaddr);

        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let soft_constraint =
            if self.child_mode {
                "ETAT='EN'"
            } else {
                ""
            }
            .to_string();

        self.question_filter = Some(
            if self.child_mode {
                "cat=1"
            } else {
                ""
            }
            .to_string()
        );

        let params = [
            (
                "callback",
                format!("jQuery331023608747682107778_{}", self.timestamp),
            ),
            ("urlApiWs", get_field!(self.ws_url)),
            ("partner", 1.to_string()),
            ("childMod", self.child_mode.to_string()),
            ("player", "website-desktop".to_string()),
            ("uid_ext_session", get_field!(self.uid)),
            ("frontaddr", get_field!(self.frontaddr)),
            ("constraint", "ETAT<>'AV'".to_string()),
            ("soft_constraint", soft_constraint),
            (
                "question_filter",
                get_field!(self.question_filter),
            ),
        ];

        let response = self.http_client
            .get(format!("{}/new_session", &self.uri))
            .headers(HEADERS.clone())
            .query(&params)
            .send()
            .await?;

        let json_string = Self::parse_response(response.text().await?);
        let json: models::StartJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_start_info(&json)?;

            Ok(self.current_question.clone())
        } else {
            Err(Self::handle_error_response(json.completion))
        }
    }

    /// answers the akinator's current question which can be retrieved with [`Self.current_question`]
    ///
    /// # Errors
    ///
    /// see [errors](https://docs.rs/akinator-rs/latest/akinator_rs/error/enum.Error.html) docs for more info
    pub async fn answer(&mut self, answer: Answer) -> Result<Option<String>> {
        let params = [
            (
                "callback",
                format!("jQuery331023608747682107778_{}", self.timestamp),
            ),
            ("urlApiWs", get_field!(self.ws_url)),
            ("childMod", self.child_mode.to_string()),
            ("session", get_field!(self.session)),
            ("signature", get_field!(self.signature)),
            ("frontaddr", get_field!(self.frontaddr)),
            ("step", self.step.to_string()),
            ("answer", (answer as u8).to_string()),
            (
                "question_filter",
                get_field!(self.question_filter),
            ),
        ];

        let response = self.http_client
            .get(format!("{}/answer_api", &self.uri))
            .headers(HEADERS.clone())
            .query(&params)
            .send()
            .await?
            .text()
            .await?;

        let json_string = Self::parse_response(response);
        let json: models::MoveJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_move_info(json)?;

            Ok(self.current_question.clone())
        } else {
            Err(Self::handle_error_response(json.completion))
        }
    }

    /// tells the akinator to end the game and make it's guess
    /// and returns its best guess, which also can be retrieved with [`Self.first_guess`]
    ///
    /// # Errors
    ///
    /// see [errors](https://docs.rs/akinator-rs/latest/akinator_rs/error/enum.Error.html) docs for more info
    pub async fn win(&mut self) -> Result<Option<models::Guess>> {
        let params = [
            (
                "callback",
                format!("jQuery331023608747682107778_{}", self.timestamp),
            ),
            ("childMod", self.child_mode.to_string()),
            ("session", get_field!(self.session)),
            ("signature", get_field!(self.signature)),
            ("step", self.step.to_string()),
        ];

        let response = self.http_client
            .get(format!("{}/list", get_field!(self.ws_url)))
            .headers(HEADERS.clone())
            .query(&params)
            .send()
            .await?
            .text()
            .await?;

        let json_string = Self::parse_response(response);
        let json: models::WinJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            let elements = json.parameters
                .ok_or(UpdateInfoError::MissingData)?
                .elements;

            self.guesses = elements
                .into_iter()
                .map(|e| e.element)
                .collect::<Vec<models::Guess>>();

            self.first_guess = self.guesses
                .first()
                .cloned();

            Ok(self.first_guess.clone())
        } else {
            Err(Self::handle_error_response(json.completion))
        }
    }

    /// Goes back 1 question and returns the current question
    /// Returns an Err value with [`Error::CantGoBackAnyFurther`] if we are already on question 0
    ///
    /// # Errors
    ///
    /// see [errors](https://docs.rs/akinator-rs/latest/akinator_rs/error/enum.Error.html) docs for more info
    pub async fn back(&mut self) -> Result<Option<String>> {
        if self.step == 0 {
            return Err(Error::CantGoBackAnyFurther);
        }

        let params = [
            (
                "callback",
                format!("jQuery331023608747682107778_{}", self.timestamp),
            ),
            ("childMod", self.child_mode.to_string()),
            ("session", get_field!(self.session)),
            ("signature", get_field!(self.signature)),
            ("step", self.step.to_string()),
            ("answer", "-1".to_string()),
            (
                "question_filter",
                get_field!(self.question_filter)
            ),
        ];

        let response = self.http_client
            .get(format!("{}/cancel_answer", get_field!(self.ws_url)))
            .headers(HEADERS.clone())
            .query(&params)
            .send()
            .await?
            .text()
            .await?;

        let json_string = Self::parse_response(response);
        let json: models::MoveJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_move_info(json)?;

            Ok(self.current_question.clone())
        } else {
            Err(Self::handle_error_response(json.completion))
        }
    }
}