use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDoc {
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub number: u32,
    pub stem: Vec<Segment>,
    pub options: Vec<OptionItem>,
    pub correct_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionItem {
    pub label: String,
    pub locked: bool,
    pub content: Vec<Segment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Segment {
    Text { text: String },
    Image { asset_path: String },
    Math { omml: String },
}
