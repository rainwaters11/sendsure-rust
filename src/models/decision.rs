use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Decision {
    Ready,
    Review,
    Stop,
}

impl std::fmt::Display for Decision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Decision::Ready => "READY",
            Decision::Review => "REVIEW",
            Decision::Stop => "STOP",
        })
    }
}

impl Decision {
    pub(crate) fn priority(self) -> u8 {
        match self {
            Decision::Ready => 0,
            Decision::Review => 1,
            Decision::Stop => 2,
        }
    }
}
