use std::time::Duration;

#[derive(Debug)]
pub enum StreamEvent {
    AdBreakBegin { duration: Duration },
    ChangeFont { username: String, font: String },
    ChangeHelixTheme { username: String, theme: String },
    ChatMessage { username: String },
    Unknown,
}

impl StreamEvent {
    pub fn new(title: &str, username: String, viewer_input: String) -> Self {
        match title {
            "change helix theme" => Self::ChangeHelixTheme {
                username,
                theme: viewer_input,
            },
            "change terminal font" => Self::ChangeFont {
                username,
                font: viewer_input,
            },
            _ => Self::Unknown,
        }
    }

    pub fn new_ad_break(duration: Duration) -> Self {
        Self::AdBreakBegin { duration }
    }
}
