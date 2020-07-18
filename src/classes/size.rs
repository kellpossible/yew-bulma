#[derive(PartialEq, Clone, Debug)]
#[allow(dead_code)]
pub enum Size {
    Small,
    Normal,
    Medium,
    Big,
}

impl Size {
    pub fn to_class(&self) -> Option<String> {
        match self {
            Size::Small => Some("is-small".to_string()),
            Size::Normal => None,
            Size::Medium => Some("is-medium".to_string()),
            Size::Big => Some("is-big".to_string()),
        }
    }
}

impl Default for Size {
    fn default() -> Self {
        Size::Normal
    }
}
