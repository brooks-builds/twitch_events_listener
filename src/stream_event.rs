#[derive(Debug)]
pub enum StreamEvent {
    ChangeHelixTheme { username: String, theme: String },
    ChangeFont { username: String, font: String },
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
}
