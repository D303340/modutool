use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SchindlerTestData {
    pub time: String,
    pub level: String,
    pub message: String
}

pub struct _SchindlerData {
    pub time: String,
    pub level: String,
    pub message: String,
}