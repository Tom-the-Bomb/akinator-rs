//! A simple wrapper crate around the Akinator API

use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use lazy_static::lazy_static;
use regex::RegexBuilder;
use reqwest::{
    blocking::Client,
    header::{
        HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, USER_AGENT,
    },
};
use serde_json::Value;

pub mod models;
pub mod enums;

lazy_static! {
    static ref HEADERS: HeaderMap<HeaderValue> = {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8"
                .parse().unwrap(),
        );
        headers.insert(ACCEPT_ENCODING, "gzip, deflate".parse().unwrap());
        headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
        headers.insert(
            USER_AGENT,
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) snap Chromium/81.0.4044.92 Chrome/81.0.4044.92 Safari/537.36"
                .parse().unwrap(),
        );
        headers.insert(
            HeaderName::from_static("x-requested-with"),
            "XMLHttpRequest".parse().unwrap(),
        );
        headers
    };
}

#[derive(Debug, Clone)]
pub struct Akinator {
    pub language: String,
    pub theme: enums::Theme,
    pub child_mode: bool,

    http_client: Client,
    timestamp: f32,
    uri: String,
    uid: Option<String>,
    ws_url: Option<String>,
    session: Option<usize>,
    frontaddr: Option<String>,
    signature: Option<usize>,
    question_filter: Option<String>,

    pub current_question: Option<String>,
    pub progression: f32,
    pub step: usize,

    pub first_guess: Option<models::Guess>,
    pub guesses: Vec<models::Guess>,
}

impl Akinator {
    pub fn new() -> Self {
        Self {
            language: "en".to_string(),
            theme: enums::Theme::Characters,
            child_mode: false,

            http_client: Client::new(),
            timestamp: 0.0,
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

    pub fn theme(mut self, theme: enums::Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn language(mut self, language: String) -> Self {
        self.language = language;
        self
    }

    pub fn with_child_mode(mut self) -> Self {
        self.child_mode = true;
        self
    }

    fn find_server(&self) -> Result<String, Box<dyn Error>> {
        let data_regex = RegexBuilder::new(
            r#"\[\{"translated_theme_name":".*","urlWs":"https:\\/\\/srv[0-9]+\.akinator\.com:[0-9]+\\/ws","subject_id":"[0-9]+"\}\]"#
        )
            .case_insensitive(true)
            .multi_line(true)
            .build()?;

        let html = self.http_client.get(&self.uri).send()?.text()?;

        let id = (self.theme.clone() as usize)
            .to_string();

        if let Some(mat) = data_regex.find(html.as_str()) {
            match serde_json::from_str(mat.as_str())? {
                Value::Array(arr) => {
                    let mat = arr
                        .iter()
                        .filter(|entry| entry["subject_id"].as_str() == Some(id.as_str()))
                        .collect::<Vec<_>>()[0];

                    return Ok(mat["urlWs"].to_string());
                }
                _ => return Err("Expected array from parsed json".into()),
            }
        }

        Err("Could not find the server uri".into())
    }

    fn find_session_info(&self) -> Result<(String, String), Box<dyn Error>> {
        let vars_regex =
            RegexBuilder::new(r#"var uid_ext_session = '(.*)';\n.*var frontaddr = '(.*)';"#)
                .case_insensitive(true)
                .multi_line(true)
                .build()?;

        let html = self
            .http_client
            .get("https://en.akinator.com/game")
            .send()?
            .text()?;

        if let Some(mat) = vars_regex.captures(html.as_str()) {
            return Ok((mat[0].to_string(), mat[1].to_string()));
        }

        Err("Could not found the session info".into())
    }

    fn parse_response(&self, html: String) -> Result<String, Box<dyn Error>> {
        let mut splits = html.split("(").collect::<Vec<_>>();

        splits.remove(0);

        let json_string = splits.join(",")
            .trim_end_matches(')')
            .to_string();

        Ok(json_string)
    }

    fn update_move_info(&mut self, json: models::MoveJson) -> Result<(), Box<dyn Error>> {
        let params = json.parameters;

        self.current_question = Some(
            params.question
        );

        self.progression = params.progression
            .parse::<f32>()?;

        self.step = params.step
            .parse::<usize>()?;

        Ok(())
    }

    fn update_start_info(&mut self, json: models::StartJson) -> Result<(), Box<dyn Error>> {
        let ident = json.parameters.identification;
        let step_info = json.parameters.step_information;

        self.session = Some(
            ident.session
                .parse::<usize>()?
        );

        self.signature = Some(
            ident.signature
                .parse::<usize>()?
        );

        self.current_question = Some(
            step_info.question
        );

        self.progression = step_info.progression
            .parse::<f32>()?;

        self.step = step_info.step
            .parse::<usize>()?;

        Ok(())
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        self.ws_url = Some(self.find_server()?);
        self.uri = format!("https://{}.akinator.com", self.language);

        let (uid, frontaddr) = self.find_session_info()?;
        self.uid = Some(uid);
        self.frontaddr = Some(frontaddr);

        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs_f32();

        let soft_constraint = match self.child_mode {
            true => "ETAT%3D%27EN%27",
            false => "",
        }
        .to_string();

        self.question_filter = Some(
            match self.child_mode {
                true => "cat%3D1",
                false => ",",
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
            .form(&params)
            .send()?
            .text()?;

        let json_string = self.parse_response(response)?;
        let json: models::StartJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_start_info(json)?;

            return Ok(());
        }

        Err("Could not connect to Akinator servers".into())
    }

    pub fn answer(&mut self, answer: enums::Answer) -> Result<&Option<String>, Box<dyn Error>> {
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
            .form(&params)
            .send()?
            .text()?;

        let json_string = self.parse_response(response)?;
        let json: models::MoveJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_move_info(json)?;
            return Ok(&self.current_question);
        }

        Err("Could not connect to Akinator servers".into())
    }

    pub fn win(&mut self) -> Result<&Option<models::Guess>, Box<dyn Error>> {
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
            .form(&params)
            .send()?
            .text()?;

        let json_string = self.parse_response(response)?;
        let json: models::WinJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            let elements = json.parameters.elements;

            self.guesses = elements
                .into_iter()
                .map(|e| e.element)
                .collect::<Vec<models::Guess>>();

            self.first_guess = self.guesses
                .first()
                .cloned();

            return Ok(&self.first_guess);
        }

        Err("Could not connect to Akinator servers".into())
    }

    pub fn back(&mut self) -> Result<(), Box<dyn Error>> {

        if self.step == 0 {
            return Err("Cannot go back any further".into())
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
            .form(&params)
            .send()?
            .text()?;

        let json_string = self.parse_response(response)?;
        let json: models::MoveJson =
            serde_json::from_str(json_string.as_str())?;

        if json.completion.as_str() == "OK" {
            self.update_move_info(json)?;

            return Ok(());
        }

        Err("Could not connect to Akinator servers".into())
    }
}