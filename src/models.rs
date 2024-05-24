use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StepInfo {
    pub step: String,
    pub question: String,
    pub progression: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IdentJson {
    pub session: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ParametersJson {
    pub identification: IdentJson,
    pub(crate) step_information: StepInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StartJson {
    pub completion: String,
    pub(crate) parameters: Option<ParametersJson>,
}


#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct MoveJson {
    pub completion: String,
    pub(crate) parameters: Option<StepInfo>,
}


#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct WinElement {
    pub element: Guess,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct WinParams {
    pub(crate) elements: Vec<WinElement>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct WinJson {
    pub completion: String,
    pub(crate) parameters: Option<WinParams>,
}


#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ServerData {
    #[serde(rename = "urlWs")]
    pub url_ws: String,
    pub subject_id: String,
}

/// represents a guess that the akinator makes at the end of the game
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guess {
    /// the unique id of the guess
    pub id: String,
    /// the name of the guess
    pub name: String,
    pub award_id: String,
    pub flag_photo: usize,
    /// the akinator's confidence level / probability that this guess is accurate
    #[serde(rename = "proba")]
    pub confidence: String,
    /// a brief desription of the guess
    pub description: String,
    /// the ranking place of the guess
    pub ranking: String,
    /// the relative url to the image of the guess
    pub picture_path: String,
    /// the absolute url to the image of the guess
    pub absolute_picture_path: String,
}