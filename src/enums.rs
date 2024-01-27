use std::{
    fmt,
    str::FromStr,
};

use crate::error::{Result, Error};


/// Enum representing a user's answer to the akinator's questions
///
/// intended to be passed into [`Akinator::answer`]
///
/// for parsing from a string, use the `from_str` / [`str::parse`] or `try_from` methods
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Yes = 0,
    No = 1,
    Idk = 2,
    Probably = 3,
    ProbablyNot = 4,
}

/// Enum representing the theme of the akinator game
///
/// intended to be pased into [`Akinator::with_theme`] when setting the theme of the game
///
/// for parsing from a string, use the `from_str` / [`str::parse`] or `from` methods
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Characters = 1,
    Animals = 14,
    Objects = 2,
}

/// Enum representing the language of the akinator game
///
/// intended to be pased into [`Akinator::with_language`] when setting the language of the game
///
/// for parsing from a string, use the `from_str` / [`str::parse`] or `try_from` methods
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Arabic,
    Chinese,
    German,
    Spanish,
    French,
    Hebrew,
    Italian,
    Japanese,
    Korean,
    Dutch,
    Polish,
    Portugese,
    Russian,
    Turkish,
    Indonesian,
}

/// internal method attempting to convert a string answer: (ex: "yes")
/// to an [`Answer`] variant
///
/// used in [`FromStr`] and [`TryFrom`] implementations
#[allow(clippy::needless_pass_by_value)]
fn try_answer_from_string(ans: String) -> Result<Answer> {
    match ans.trim().to_lowercase().as_str() {
        "yes" | "y" | "0" => Ok(Answer::Yes),
        "no"  | "n" | "1" => Ok(Answer::No),
        "i dont know" | "i don't know" | "idk" | "i" | "2" => Ok(Answer::Idk),
        "probably" | "p" | "3" => Ok(Answer::Probably),
        "probably not" | "pn" | "4" => Ok(Answer::ProbablyNot),
        _ => Err(Error::InvalidAnswer),
    }
}

impl FromStr for Answer {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        try_answer_from_string(string.to_string())
    }
}

impl TryFrom<&str> for Answer {
    type Error = Error;

    fn try_from(ans: &str) -> Result<Self, Self::Error> {
        try_answer_from_string(ans.to_string())
    }
}

impl TryFrom<String> for Answer {
    type Error = Error;

    fn try_from(ans: String) -> Result<Self, Self::Error> {
        try_answer_from_string(ans)
    }
}

impl TryFrom<usize> for Answer {
    type Error = Error;

    fn try_from(ans: usize) -> Result<Self, Self::Error> {
        try_answer_from_string(ans.to_string())
    }
}

/// internal method to convert a string representing a theme: (ex: "animals")
/// to a [`Theme`] variant
///
/// used in [`FromStr`] and [`From`] implementations
#[allow(clippy::needless_pass_by_value)]
fn theme_from_string(theme: String) -> Theme {
    match theme.trim().to_lowercase().as_str() {
        "a" | "animals" => Theme::Animals,
        "o" | "objects" => Theme::Objects,
        _ => Theme::default(),
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::Characters
    }
}

impl FromStr for Theme {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Ok(theme_from_string(string.to_string()))
    }
}

impl From<&str> for Theme {
    fn from(theme: &str) -> Self {
        theme_from_string(theme.to_string())
    }
}

impl From<String> for Theme {
    fn from(theme: String) -> Self {
        theme_from_string(theme)
    }
}

impl From<usize> for Theme {
    fn from(theme: usize) -> Self {
        theme_from_string(theme.to_string())
    }
}


impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::English => "en",
            Self::Arabic => "ar",
            Self::Chinese => "cn",
            Self::German => "de",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::Hebrew => "il",
            Self::Italian => "it",
            Self::Japanese => "jp",
            Self::Korean => "kr",
            Self::Dutch => "nl",
            Self::Polish => "pl",
            Self::Portugese => "pt",
            Self::Russian => "ru",
            Self::Turkish => "tr",
            Self::Indonesian => "id",
        })
    }
}

/// internal method attempting to convert a string representing a language: (ex: "english")
/// to a [`Language`] variant
///
/// used in [`FromStr`] and [`TryFrom`] implementations
#[allow(clippy::needless_pass_by_value)]
fn try_lang_from_string(lang: String) -> Result<Language> {
    match lang.trim().to_lowercase().as_str() {
        "english" | "en" => Ok(Language::English),
        "arabic"  | "ar" => Ok(Language::Arabic),
        "chinese" | "cn" => Ok(Language::Chinese),
        "spanish" | "es" => Ok(Language::Spanish),
        "french"  | "fr" => Ok(Language::French),
        "hebrew"  | "il" => Ok(Language::Hebrew),
        "italian" | "it" => Ok(Language::Italian),
        "japanese" | "jp" => Ok(Language::Japanese),
        "korean"  | "kr" => Ok(Language::Korean),
        "dutch"  | "nl" => Ok(Language::Dutch),
        "polish" | "pl" => Ok(Language::Polish),
        "portugese" | "pt" => Ok(Language:: Portugese),
        "russian" | "ru" => Ok(Language::Russian),
        "turkish" | "tr" => Ok(Language::Turkish),
        "indonesian" | "id" => Ok(Language::Indonesian),
        "german" | "de" => Ok(Language::German),
        _ => Err(Error::InvalidLanguage)
    }
}

impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}

impl FromStr for Language {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        try_lang_from_string(string.to_string())
    }
}

impl TryFrom<&str> for Language {
    type Error = Error;

    fn try_from(ans: &str) -> Result<Self, Self::Error> {
        try_lang_from_string(ans.to_string())
    }
}


impl TryFrom<String> for Language {
    type Error = Error;

    fn try_from(ans: String) -> Result<Self, Self::Error> {
        try_lang_from_string(ans)
    }
}