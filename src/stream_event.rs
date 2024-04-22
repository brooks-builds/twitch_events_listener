#[derive(Debug)]
pub enum StreamEvent {
    ChangeHelixTheme { username: String, theme: String },
    Unknown,
}

impl StreamEvent {
    pub fn new(title: &str, username: String, theme: String) -> Self {
        match title {
            "change helix theme" => Self::ChangeHelixTheme { username, theme },
            _ => Self::Unknown,
        }
    }
}
