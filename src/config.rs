#[derive(serde::Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    #[serde(rename = "doneSentences")]
    pub done_sentences: Vec<String>,
    /// Show done sentece for x seconds
    #[serde(rename = "doneShowTime")]
    pub done_show_time: u32,

    #[serde(rename = "bgColor")]
    pub bg_color: u32,
    #[serde(rename = "errorBgColor")]
    pub error_bg_color: u32,

    #[serde(rename = "textColor")]
    pub text_color: u32,

    #[serde(rename = "takePictureText")]
    pub take_picture_text: String,

    /// The default text size
    #[serde(rename = "textSize")]
    pub text_size: f32,

    /// Countdown in seconds
    pub countdown: u32,
    #[serde(rename = "countdownTextSize")]
    pub countdown_text_size: f32,

    /// Show the resulting image for (minimum) x seconds
    #[serde(rename = "showImageTime")]
    pub show_image_time: u32,

    /// The sub path on the USB device where the images should be saved
    #[serde(rename = "storageSubPath")]
    pub storage_sub_path: Option<String>,

    /// Display error messages for x time
    #[serde(rename = "errorMessageTime")]
    pub error_message_time: u32,
    #[serde(rename = "unknownErrorMessage")]
    pub unknown_error_message: String,
    #[serde(rename = "errorNoUsbDevice")]
    pub error_no_usb_device: String,
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
            done_show_time: 5,
            bg_color: 0xFF32a8a8,
            error_bg_color: 0xFFed4e4e,
            text_color: 0x00FFFFFF,
            take_picture_text: "Touch to take a picture".to_string(),
            text_size: 100.,
            countdown_text_size: 350.,
            countdown: 5,
            show_image_time: 5,
            storage_sub_path: None,
            error_message_time: 8,
            unknown_error_message: "Unkown error".to_string(),
            error_no_usb_device: "No USB device connected".to_string(),
        }
    }
}
