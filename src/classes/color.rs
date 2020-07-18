#[derive(PartialEq, Clone, Debug)]
#[allow(dead_code)]
pub enum Color {
    White,
    Black,
    Light,
    Dark,
    Primary,
    Info,
    Link,
    Success,
    Warning,
    Danger,
    BlackBis,
    BlackTer,
    GreyDarker,
    GreyDark,
    Grey,
    GreyLight,
    WhiteTer,
    WhiteBis,
}

#[allow(dead_code)]
impl Color {
    pub fn to_str(&self) -> &str {
        match self {
            Color::White => "white",
            Color::Black => "black",
            Color::Light => "light",
            Color::Dark => "dark",
            Color::Primary => "primary",
            Color::Info => "info",
            Color::Link => "link",
            Color::Success => "success",
            Color::Warning => "warning",
            Color::Danger => "danger",
            Color::BlackBis => "black-bis",
            Color::BlackTer => "black-ter",
            Color::GreyDarker => "grey-darker",
            Color::GreyDark => "grey-dark",
            Color::Grey => "grey",
            Color::GreyLight => "grey-light",
            Color::WhiteTer => "white-ter",
            Color::WhiteBis => "white-bis",
        }
    }

    pub fn text_class(&self) -> String {
        format!("has-text-{0}", self.to_str())
    }

    pub fn background_class(&self) -> String {
        format!("has-background-{0}", self.to_str())
    }

    pub fn class(&self) -> String {
        format!("is-{0}", self.to_str())
    }
}
