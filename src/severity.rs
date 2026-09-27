/// Severity level for insights and AUI rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityLevel {
    Critical,
    Error,
    Warning,
    Info,
    Note,
}

impl SeverityLevel {
    /// Parse severity from string
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Self::Critical,
            "error" => Self::Error,
            "warning" => Self::Warning,
            "info" => Self::Info,
            _ => Self::Note,
        }
    }

    /// Get display indicator for DIM observation rendering
    #[must_use]
    pub const fn indicator(&self) -> &'static str {
        match self {
            Self::Critical => "[CRITICAL]",
            Self::Error => "[error]",
            Self::Warning => "[warning]",
            Self::Info => "[info]",
            Self::Note => "[note]",
        }
    }
}
