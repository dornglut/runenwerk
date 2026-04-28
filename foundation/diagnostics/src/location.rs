use core::fmt;

#[cfg(feature = "alloc")]
use alloc::string::String;

/// Structured diagnostic location.
///
/// Locations are generic and not limited to source-code spans.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DiagnosticLocation {
    TextRange(DiagnosticTextRange),

    FilePath {
        path: DiagnosticLocationPath,
        range: Option<DiagnosticTextRange>,
    },

    LogicalPath(DiagnosticLocationPath),

    FieldPath(DiagnosticLocationPath),
}

/// One-based text position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticTextPosition {
    line: u32,
    column: u32,
}

/// Inclusive-start, exclusive-end text range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticTextRange {
    start: DiagnosticTextPosition,
    end: DiagnosticTextPosition,
}

/// Public diagnostic path value.
///
/// This keeps `DiagnosticLocation` variants public without exposing the
/// internal storage representation used for static versus owned paths.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticLocationPath {
    value: LocationStorage,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LocationStorage {
    Static(&'static str),

    #[cfg(feature = "alloc")]
    Owned(String),
}

/// Error returned when constructing an invalid diagnostic location.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticLocationError {
    EmptyPath,
    InvalidPosition,
    InvalidRange,
}

impl DiagnosticLocation {
    pub fn text_range(range: DiagnosticTextRange) -> Self {
        Self::TextRange(range)
    }

    pub fn file_path_static(
        path: &'static str,
        range: Option<DiagnosticTextRange>,
    ) -> Result<Self, DiagnosticLocationError> {
        if path.is_empty() {
            return Err(DiagnosticLocationError::EmptyPath);
        }

        Ok(Self::FilePath {
            path: DiagnosticLocationPath::from_static_unchecked(path),
            range,
        })
    }

    #[cfg(feature = "alloc")]
    pub fn file_path(
        path: impl Into<String>,
        range: Option<DiagnosticTextRange>,
    ) -> Result<Self, DiagnosticLocationError> {
        let path = path.into();

        if path.is_empty() {
            return Err(DiagnosticLocationError::EmptyPath);
        }

        Ok(Self::FilePath {
            path: DiagnosticLocationPath::from_owned_unchecked(path),
            range,
        })
    }

    pub fn logical_path_static(path: &'static str) -> Result<Self, DiagnosticLocationError> {
        if path.is_empty() {
            return Err(DiagnosticLocationError::EmptyPath);
        }

        Ok(Self::LogicalPath(
            DiagnosticLocationPath::from_static_unchecked(path),
        ))
    }

    #[cfg(feature = "alloc")]
    pub fn logical_path(path: impl Into<String>) -> Result<Self, DiagnosticLocationError> {
        let path = path.into();

        if path.is_empty() {
            return Err(DiagnosticLocationError::EmptyPath);
        }

        Ok(Self::LogicalPath(
            DiagnosticLocationPath::from_owned_unchecked(path),
        ))
    }

    pub fn field_path_static(path: &'static str) -> Result<Self, DiagnosticLocationError> {
        if path.is_empty() {
            return Err(DiagnosticLocationError::EmptyPath);
        }

        Ok(Self::FieldPath(
            DiagnosticLocationPath::from_static_unchecked(path),
        ))
    }

    #[cfg(feature = "alloc")]
    pub fn field_path(path: impl Into<String>) -> Result<Self, DiagnosticLocationError> {
        let path = path.into();

        if path.is_empty() {
            return Err(DiagnosticLocationError::EmptyPath);
        }

        Ok(Self::FieldPath(
            DiagnosticLocationPath::from_owned_unchecked(path),
        ))
    }
}

impl DiagnosticTextPosition {
    pub const fn new(line: u32, column: u32) -> Result<Self, DiagnosticLocationError> {
        if line == 0 || column == 0 {
            return Err(DiagnosticLocationError::InvalidPosition);
        }

        Ok(Self { line, column })
    }

    pub const fn line(self) -> u32 {
        self.line
    }

    pub const fn column(self) -> u32 {
        self.column
    }
}

impl DiagnosticTextRange {
    pub const fn new(
        start: DiagnosticTextPosition,
        end: DiagnosticTextPosition,
    ) -> Result<Self, DiagnosticLocationError> {
        if start.line > end.line {
            return Err(DiagnosticLocationError::InvalidRange);
        }

        if start.line == end.line && start.column >= end.column {
            return Err(DiagnosticLocationError::InvalidRange);
        }

        Ok(Self { start, end })
    }

    pub const fn start(self) -> DiagnosticTextPosition {
        self.start
    }

    pub const fn end(self) -> DiagnosticTextPosition {
        self.end
    }
}

impl LocationStorage {
    fn as_str(&self) -> &str {
        match self {
            LocationStorage::Static(value) => value,
            #[cfg(feature = "alloc")]
            LocationStorage::Owned(value) => value.as_str(),
        }
    }
}

impl DiagnosticLocationPath {
    pub fn from_static(path: &'static str) -> Result<Self, DiagnosticLocationError> {
        if path.is_empty() {
            return Err(DiagnosticLocationError::EmptyPath);
        }

        Ok(Self::from_static_unchecked(path))
    }

    #[cfg(feature = "alloc")]
    pub fn new(path: impl Into<String>) -> Result<Self, DiagnosticLocationError> {
        let path = path.into();

        if path.is_empty() {
            return Err(DiagnosticLocationError::EmptyPath);
        }

        Ok(Self::from_owned_unchecked(path))
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }

    fn from_static_unchecked(path: &'static str) -> Self {
        Self {
            value: LocationStorage::Static(path),
        }
    }

    #[cfg(feature = "alloc")]
    fn from_owned_unchecked(path: String) -> Self {
        Self {
            value: LocationStorage::Owned(path),
        }
    }
}

impl fmt::Display for DiagnosticTextPosition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.line, self.column)
    }
}

impl fmt::Display for DiagnosticTextRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}..{}", self.start, self.end)
    }
}

impl fmt::Display for DiagnosticLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticLocation::TextRange(range) => write!(formatter, "{range}"),
            DiagnosticLocation::FilePath { path, range } => match range {
                Some(range) => write!(formatter, "{}:{range}", path.as_str()),
                None => formatter.write_str(path.as_str()),
            },
            DiagnosticLocation::LogicalPath(path) => formatter.write_str(path.as_str()),
            DiagnosticLocation::FieldPath(path) => formatter.write_str(path.as_str()),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticTextPosition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("DiagnosticTextPosition", 2)?;
        state.serialize_field("line", &self.line)?;
        state.serialize_field("column", &self.column)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticTextPosition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Position {
            line: u32,
            column: u32,
        }

        let position = Position::deserialize(deserializer)?;

        Self::new(position.line, position.column).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid diagnostic text position: {error:?}"))
        })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticTextRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut state = serializer.serialize_struct("DiagnosticTextRange", 2)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticTextRange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Range {
            start: DiagnosticTextPosition,
            end: DiagnosticTextPosition,
        }

        let range = Range::deserialize(deserializer)?;

        Self::new(range.start, range.end).map_err(|error| {
            serde::de::Error::custom(format_args!("invalid diagnostic text range: {error:?}"))
        })
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for LocationStorage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LocationStorage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        if value.is_empty() {
            return Err(serde::de::Error::custom(
                "diagnostic location path is empty",
            ));
        }

        Ok(Self::Owned(value))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DiagnosticLocation, DiagnosticLocationError, DiagnosticLocationPath,
        DiagnosticTextPosition, DiagnosticTextRange,
    };

    #[test]
    fn location_rejects_invalid_range() {
        let start = DiagnosticTextPosition::new(2, 1).unwrap();
        let end = DiagnosticTextPosition::new(1, 1).unwrap();

        assert_eq!(
            DiagnosticTextRange::new(start, end),
            Err(DiagnosticLocationError::InvalidRange)
        );
    }

    #[test]
    fn location_rejects_zero_position() {
        assert_eq!(
            DiagnosticTextPosition::new(0, 1),
            Err(DiagnosticLocationError::InvalidPosition)
        );
    }

    #[test]
    fn location_can_represent_file_path() {
        let start = DiagnosticTextPosition::new(1, 1).unwrap();
        let end = DiagnosticTextPosition::new(1, 5).unwrap();
        let range = DiagnosticTextRange::new(start, end).unwrap();

        let location =
            DiagnosticLocation::file_path_static("assets/scene.rwscene", Some(range)).unwrap();

        assert_eq!(location.to_string(), "assets/scene.rwscene:1:1..1:5");
    }

    #[test]
    fn location_can_represent_field_path() {
        let location = DiagnosticLocation::field_path_static("entities[4].parent").unwrap();

        assert_eq!(location.to_string(), "entities[4].parent");
    }

    #[test]
    fn location_can_represent_logical_path() {
        let location =
            DiagnosticLocation::logical_path_static("workspace.tool_surfaces[2]").unwrap();

        assert_eq!(location.to_string(), "workspace.tool_surfaces[2]");
    }

    #[test]
    fn empty_location_path_is_rejected() {
        assert_eq!(
            DiagnosticLocation::field_path_static(""),
            Err(DiagnosticLocationError::EmptyPath)
        );
    }

    #[test]
    fn diagnostic_location_path_rejects_empty_path() {
        assert_eq!(
            DiagnosticLocationPath::from_static(""),
            Err(DiagnosticLocationError::EmptyPath)
        );
    }

    #[test]
    fn diagnostic_location_path_preserves_public_path_access() {
        let path = DiagnosticLocationPath::from_static("workspace.tool_surfaces[2]").unwrap();

        assert_eq!(path.as_str(), "workspace.tool_surfaces[2]");
    }
}
