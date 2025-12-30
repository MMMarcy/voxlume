use std::fmt::Display;

#[derive(Clone, PartialEq, Debug, Copy, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub fn from_string<S: AsRef<str>>(s: Option<S>) -> Theme {
        match s {
            Some(val) if val.as_ref() == "dark" => Theme::Dark,
            _ => Theme::Light,
        }
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Theme::Light => "light".to_string(),
            Theme::Dark => "dark".to_string(),
        };
        write!(f, "{value}")
    }
}
