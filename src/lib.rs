//! A simple wrapper crate around the Akinator API

use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;
use regex::RegexBuilder;
use reqwest::{
    Client,
    header::{
        HeaderMap, HeaderName, HeaderValue, USER_AGENT,
    },
};
use crate::enums::{Theme, Answer, Language};
use crate::error::{
    Result,
    Error,
    UpdateInfoError,
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


/// Represents an akinator game
#[derive(Debug, Clone)]
pub struct Akinator {
    /// The language for the akinator session
    pub language: Language,
    /// The theme for the akinator session
    /// One of 'Characters', 'Animals', or 'Objects'
    pub theme: Theme,
    /// indicates whether or not to filter out NSFW questions and content
    pub child_mode: bool,

    /// The reqwest client used for this akinator session
    http_client: Client,
    /// The timestamp the game session was started
    /// used for keeping track of sessions
    timestamp: u64,
    /// the base URI to use when making requests
    /// usually: https://{language}.akinator.com/
    uri: String,
    /// The unique identifier for the akinator session
    uid: Option<String>,
    /// the websocket url used for the game
    ws_url: Option<String>,
    session: Option<usize>,
    frontaddr: Option<String>,
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
    /// Only will be set when [`Self::win`] has been called
    pub first_guess: Option<models::Guess>,
    /// a vec containing all the possible guesses by the akinator
    /// Only will be set when [`Self::win`] has been called
    pub guesses: Vec<models::Guess>,
}

impl Akinator {
    /// Creates a new [`Akinator`] instance
    /// with fields filled with default values
    pub fn new() -> Self {
        Self {
            language: Language::English,
            theme: Theme::Characters,
            child_mode: false,

            http_client: Client::new(),
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
        }
    }

    /// builder method to set the [`Self.theme`] for the akinator game
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// builder method to set the [`Self.language`] for the akinator game
    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    /// builder function to turn on [`Self.child_mode`]
    pub fn with_child_mode(mut self) -> Self {
        self.child_mode = true;
        self
    }

    /// Internal method to handle an error response from the akinator API
    /// and return an appropriate Err value
    fn handle_error_response(&self, completion: String) -> Error {
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
        let data_regex = RegexBuilder::new(
            r#"\[\{"translated_theme_name":".*","urlWs":"https:\\/\\/srv[0-9]+\.akinator\.com:[0-9]+\\/ws","subject_id":"[0-9]+"\}\]"#
        )
            .case_insensitive(true)
            .multi_line(true)
            .build()?;

        let html = self.http_client.get(&self.uri)
            .send()
            .await?
            .text()
            .await?;

        let id = (self.theme.clone() as usize)
            .to_string();

        if let Some(mat) = data_regex.find(html.as_str()) {
            let json: Vec<models::ServerData> =
                serde_json::from_str(mat.as_str())?;

            let mat = json
                .iter()
                .filter(|entry| entry.subject_id == id)
                .collect::<Vec<_>>()[0];

            return Ok(mat.urlWs.clone());
        }

        Err(Error::NoDataFound)
    }

    /// internal method used to parse and find the session uid and frontaddr for the akinator session
    /// Done by parsing the javascript of the site, extracting variable values
    async fn find_session_info(&self) -> Result<(String, String)> {
        let vars_regex =
            RegexBuilder::new(r#"var uid_ext_session = '(.*)';\n.*var frontaddr = '(.*)';"#)
                .case_insensitive(true)
                .multi_line(true)
                .build()?;

        let html = self.http_client
            .get("https://en.akinator.com/game")
            .send()
            .await?
            .text()
            .await?;

        if let Some(mat) = vars_regex.captures(html.as_str()) {
            return Ok((mat[1].to_string(), mat[2].to_string()));
        }

        Err(Error::NoDataFound)
    }

    /// internal method used to parse the response returned from the API
    /// strips the function call wrapped around the json, returning the json string
    fn parse_response(&self, html: String) -> Result<String> {
        let response_regex =
            RegexBuilder::new(r"^jQuery\d+_\d+\((?P<json>\{.+\})\)$")
                .case_insensitive(true)
                .multi_line(true)
                .build()?;

        Ok(response_regex
            .replace(html.as_str(), "$json")
            .to_string())
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
    fn update_start_info(&mut self, json: models::StartJson) -> Result<(), UpdateInfoError> {
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
    pub async fn start(&mut self) -> Result<Option<String>> {
        self.ws_url = Some(self.find_server().await?);
        self.uri = format!("https://{}.akinator.com", self.language.to_string());

        let (uid, frontaddr) = self.find_session_info().await?;
        self.uid = Some(uid);
        self.frontaddr = Some(frontaddr);

        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let soft_constraint = match self.child_mode {
            true => "ETAT='EN'",
            false => "",
        }
        .to_string();

        self.question_filter = Some(
            match self.child_mode {
                true => "cat=1",
                false => "",
            }
            .to_string(),
        );

        let params = [
            (
                "callback",
                format!("jQuery331023608747682107778_{}", self.timestamp.to_string()),
            ),
            ("urlApiWs", self.ws_url.as_ref().unwrap().to_string()),
            ("partner", 1.to_string()),
            ("childMod", self.child_mode.to_string()),
            ("player", "website-desktop".to_string()),
            ("uid_ext_session", self.uid.as_ref().unwrap().to_string()),
            ("frontaddr", self.frontaddr.as_ref().unwrap().to_string()),
            ("constraint", "ETAT<>'AV'".to_string()),
            ("soft_constraint", soft_constraint),
            (
                "question_filter",
                self.question_filter.as_ref().unwrap().to_string(),
            ),
        ];

        let response = self.http_client
            .get(format!("{}/new_session", &self.uri))
            .headers(HEADERS.clone())
            .query(&params)
            .send()
            .await?;

        let json_string = self.parse_response(response.text().await?)?;
        let json: models::StartJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_start_info(json)?;

            return Ok(self.current_question.clone());
        }

        Err(self.handle_error_response(json.completion))
    }

    /// answers the akinator's current question which can be retrieved with [`Self.current_question`]
    pub async fn answer(&mut self, answer: Answer) -> Result<Option<String>> {
        let params = [
            (
                "callback",
                format!("jQuery331023608747682107778_{}", self.timestamp.to_string()),
            ),
            ("urlApiWs", self.ws_url.as_ref().unwrap().to_string()),
            ("childMod", self.child_mode.to_string()),
            ("session", self.session.as_ref().unwrap().to_string()),
            ("signature", self.signature.as_ref().unwrap().to_string()),
            ("frontaddr", self.frontaddr.as_ref().unwrap().to_string()),
            ("step", self.step.to_string()),
            ("answer", (answer as u8).to_string()),
            (
                "question_filter",
                self.question_filter.as_ref().unwrap().to_string(),
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

        let json_string = self.parse_response(response)?;
        let json: models::MoveJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_move_info(json)?;
            return Ok(self.current_question.clone());
        }

        Err(self.handle_error_response(json.completion))
    }

    /// tells the akinator to end the game and make it's guess
    /// and returns its best guess
    pub async fn win(&mut self) -> Result<Option<models::Guess>> {
        let params = [
            (
                "callback",
                format!("jQuery331023608747682107778_{}", self.timestamp.to_string()),
            ),
            ("childMod", self.child_mode.to_string()),
            ("session", self.session.as_ref().unwrap().to_string()),
            ("signature", self.signature.as_ref().unwrap().to_string()),
            ("step", self.step.to_string()),
        ];

        let response = self.http_client
            .get(format!("{}/list", self.ws_url.as_ref().unwrap()))
            .headers(HEADERS.clone())
            .query(&params)
            .send()
            .await?
            .text()
            .await?;

        let json_string = self.parse_response(response)?;
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

            return Ok(self.first_guess.clone());
        }

        Err(self.handle_error_response(json.completion))
    }

    /// Goes back 1 question and returns the current question
    /// Returns an Err value with [`Error::CantGoBackAnyFurther`] if we are already on question 0
    pub async fn back(&mut self) -> Result<Option<String>> {
        if self.step == 0 {
            return Err(Error::CantGoBackAnyFurther);
        }

        let params = [
            (
                "callback",
                format!("jQuery331023608747682107778_{}", self.timestamp.to_string()),
            ),
            ("childMod", self.child_mode.to_string()),
            ("session", self.session.as_ref().unwrap().to_string()),
            ("signature", self.signature.as_ref().unwrap().to_string()),
            ("step", self.step.to_string()),
            ("answer", "-1".to_string()),
            (
                "question_filter",
                self.question_filter.as_ref().unwrap().to_string()
            ),
        ];

        let response = self.http_client
            .get(format!("{}/cancel_answer", self.ws_url.as_ref().unwrap()))
            .headers(HEADERS.clone())
            .query(&params)
            .send()
            .await?
            .text()
            .await?;

        let json_string = self.parse_response(response)?;
        let json: models::MoveJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_move_info(json)?;

            return Ok(self.current_question.clone());
        }

        Err(self.handle_error_response(json.completion))
    }
}