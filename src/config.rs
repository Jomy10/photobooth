#[derive(serde::Deserialize)]
pub struct Config {
    #[serde(default)]
    #[serde(rename = "doneSentences")]
    pub done_sentences: Vec<String>,
    #[serde(default)]
    #[serde(rename = "bgColor")]
    pub bg_color: u32,
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
            bg_color: 0xFF32a8a8
        }
    }
}
