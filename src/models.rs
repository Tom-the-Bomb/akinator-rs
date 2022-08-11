use serde::{Serialize, Deserialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct StepInfo {
    pub step: String,
    pub question: String,
    pub progression: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct IdentJson {
    pub session: String,
    pub signature: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParametersJson {
    pub identification: IdentJson,
    pub step_information: StepInfo,
}

#[derive(Serialize, Deserialize)]
pub struct StartJson {
    pub completion: String,
    pub parameters: Option<ParametersJson>,
}


#[derive(Serialize, Deserialize)]
pub struct MoveJson {
    pub completion: String,
    pub parameters: Option<StepInfo>,
}


#[derive(Serialize, Deserialize)]
pub struct WinElement {
    pub element: Guess,
}

#[derive(Serialize, Deserialize)]
pub struct WinParams {
    pub elements: Vec<WinElement>,
}

#[derive(Serialize, Deserialize)]
pub struct WinJson {
    pub completion: String,
    pub parameters: Option<WinParams>,
}


#[derive(Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ServerData {
    pub urlWs: String,
    pub subject_id: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
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