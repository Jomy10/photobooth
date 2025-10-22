#[derive(serde::Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    #[serde(rename = "doneSentences")]
    pub done_sentences: Vec<String>,

    #[serde(rename = "bgColor")]
    pub bg_color: u32,

    #[serde(rename = "takePictureText")]
    pub take_picture_text: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            done_sentences: [
                "All done!",
                "You look great!",
                "Come closer again",
                "Looking good 😎",
                "Curious to see the result?"
            ].map(|s| s.into()).to_vec(),
            bg_color: 0xFF32a8a8,
            take_picture_text: "Touch to take a picture".to_string(),
        }
    }
}
