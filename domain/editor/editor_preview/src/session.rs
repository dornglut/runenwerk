use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PreviewSessionId(u64);

impl PreviewSessionId {
    pub const fn try_from_raw(raw: u64) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

pub const fn preview_session_id(raw: u64) -> PreviewSessionId {
    match PreviewSessionId::try_from_raw(raw) {
        Some(id) => id,
        None => panic!("preview session id constants must be non-zero"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_session_id_is_rejected() {
        assert_eq!(PreviewSessionId::try_from_raw(0), None);
        assert_eq!(preview_session_id(7).raw(), 7);
    }
}
