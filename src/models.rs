use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub(crate) struct StepInfo {
    pub step: String,
    pub question: String,
    pub progression: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct IdentJson {
    pub session: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ParametersJson {
    pub identification: IdentJson,
    pub(crate) step_information: StepInfo,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct StartJson {
    pub completion: String,
    pub(crate) parameters: Option<ParametersJson>,
}


#[derive(Serialize, Deserialize)]
pub(crate) struct MoveJson {
    pub completion: String,
    pub(crate) parameters: Option<StepInfo>,
}


#[derive(Serialize, Deserialize)]
pub(crate) struct WinElement {
    pub element: Guess,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WinParams {
    pub(crate) elements: Vec<WinElement>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct WinJson {
    pub completion: String,
    pub(crate) parameters: Option<WinParams>,
}


#[derive(Serialize, Deserialize)]
pub(crate) struct ServerData {
    #[serde(rename = "urlWs")]
    pub url_ws: String,
    pub subject_id: String,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guess {
    pub id: String,
    pub name: String,
    pub award_id: String,
    pub flag_photo: usize,
    pub description: String,
    pub ranking: String,
    pub picture_path: String,
    pub absolute_picture_path: String,
}