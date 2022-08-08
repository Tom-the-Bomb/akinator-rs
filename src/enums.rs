
#[derive(Debug)]
pub enum Answer {
    Yes = 0,
    No = 1,
    Idk = 2,
    Probably = 3,
    ProbablyNot = 4,
}

fn try_from_string(ans: String) -> Result<Answer, &'static str> {
    match ans.to_ascii_lowercase().as_str() {
        "yes" | "y" | "0" => Ok(Answer::Yes),
        "no"  | "n" | "1" => Ok(Answer::No),
        "i dont know" | "i don't know" | "idk" | "i" | "2" => Ok(Answer::Idk),
        "probably" | "p" | "3" => Ok(Answer::Probably),
        "probably not" | "pn" | "4" => Ok(Answer::ProbablyNot),
        _ => Err("Invalid answer"),
    }
}

impl TryFrom<&str> for Answer {
    type Error = &'static str;

    fn try_from(ans: &str) -> Result<Self, Self::Error> {
        try_from_string(ans.to_string())
    }
}


impl TryFrom<String> for Answer {
    type Error = &'static str;

    fn try_from(ans: String) -> Result<Self, Self::Error> {
        try_from_string(ans)
    }
}

impl TryFrom<usize> for Answer {
    type Error = &'static str;

    fn try_from(ans: usize) -> Result<Self, Self::Error> {
        try_from_string(ans.to_string())
    }
}