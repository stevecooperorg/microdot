#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Direction {
    #[default]
    TB,
    LR,
    RL,
    BT,
}

impl Direction {
    pub fn to_bool(self) -> bool {
        matches!(self, Direction::LR | Direction::RL)
    }

    pub fn from_bool(is_left_right: bool) -> Self {
        if is_left_right {
            Direction::LR
        } else {
            Direction::TB
        }
    }

    pub fn to_string(self) -> &'static str {
        match self {
            Direction::LR => "LR",
            Direction::RL => "RL",
            Direction::TB => "TB",
            Direction::BT => "BT",
        }
    }
}

