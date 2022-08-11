use std::str::FromStr;
use crate::error::{Result, Error};

#[derive(Clone, Debug)]
pub enum Answer {
    Yes = 0,
    No = 1,
    Idk = 2,
    Probably = 3,
    ProbablyNot = 4,
}

#[derive(Clone, Debug)]
pub enum Theme {
    Characters = 1,
    Animals = 14,
    Objects = 2,
}

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


fn theme_from_string(theme: String) -> Theme {
    match theme.trim().to_lowercase().as_str() {
        "a" | "animals" => Theme::Animals,
        "o" | "objects" => Theme::Objects,
        _ => Theme::Characters,
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