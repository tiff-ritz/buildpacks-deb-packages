use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub(crate) enum DistroCodename {
    Jammy,
    Noble,
}

impl Display for DistroCodename {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DistroCodename::Jammy => write!(f, "jammy"),
            DistroCodename::Noble => write!(f, "noble"),
        }
    }
}