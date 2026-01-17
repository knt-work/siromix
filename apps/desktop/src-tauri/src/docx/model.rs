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

/// A segment represents a piece of content within a question or option.
/// Each segment preserves the original XML from the DOCX file to maintain formatting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Segment {
    /// Text segment with plain text and original XML run (<w:r>...</w:r>)
    #[serde(rename = "Text")]
    Text { 
        text: String,
        #[serde(rename = "rawXml")]
        raw_xml: String,
    },
    /// Image segment with asset path and original XML drawing/object
    #[serde(rename = "Image")]
    Image { 
        asset_path: String,
        #[serde(rename = "rawXml")]
        raw_xml: String,
        /// Width in EMUs (parsed from original XML, 0 if not found)
        #[serde(rename = "widthEmu", default)]
        width_emu: i64,
        /// Height in EMUs (parsed from original XML, 0 if not found)
        #[serde(rename = "heightEmu", default)]
        height_emu: i64,
    },
    /// Math segment with OMML content and original XML wrapper
    #[serde(rename = "Math")]
    Math { 
        omml: String,
        #[serde(rename = "rawXml")]
        raw_xml: String,
    },
}
