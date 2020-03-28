use std::fmt;

pub enum EnvironmentStatus {
    Okay,
    Stale,
    Unknown,
}

impl fmt::Display for EnvironmentStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EnvironmentStatus::*;
        match self {
            Okay => write!(f, "Environment is up to date!"),
            Stale => write!(f, "Environment is STALE!"),
            Unknown => write!(f, "Environment not built or otherwise broken!"),
        }
    }
}

impl EnvironmentStatus {
    pub fn display(&self) -> String {
        format!("{}", self)
    }

    pub fn code(&self) -> u8 {
        use EnvironmentStatus::*;
        match self {
            Okay => 0,
            Stale => 1,
            Unknown => 2,
        }
    }
}
