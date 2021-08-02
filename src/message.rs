use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SqipCreateMessage {
    pub film_id: String,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SqipDoneMessage {
    pub film_id: String,
    pub poster_svg_base64encoded: Option<String>,
    pub backdrop_svg_base64encoded: Option<String>,
}

impl SqipDoneMessage {
    pub fn new(film_id: String) -> Self {
        Self {
            film_id,
            poster_svg_base64encoded: None,
            backdrop_svg_base64encoded: None,
        }
    }
}
