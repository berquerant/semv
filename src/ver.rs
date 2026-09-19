use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::json;

fn trim_prefix_v(s: &str) -> &str {
    if let Some(x) = s.strip_prefix('v') {
        x
    } else {
        s
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct VersionInfo {
    pub original: String,
    pub version: Option<Version>,
}

impl VersionInfo {
    pub fn parse(s: &str) -> VersionInfo {
        let v = Version::parse(s)
            .or_else(|_| Version::parse(trim_prefix_v(s)))
            .ok();
        VersionInfo {
            original: s.to_string(),
            version: v,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ParsedSemver {
    pub original: String,
    pub is_valid: bool,
    pub major: Option<u64>,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub pre: Option<String>,
    pub build: Option<String>,
}

impl From<&VersionInfo> for ParsedSemver {
    fn from(info: &VersionInfo) -> Self {
        if let Some(v) = &info.version {
            ParsedSemver {
                original: info.original.clone(),
                is_valid: true,
                major: Some(v.major),
                minor: Some(v.minor),
                patch: Some(v.patch),
                pre: if v.pre.is_empty() {
                    None
                } else {
                    Some(v.pre.as_str().to_string())
                },
                build: if v.build.is_empty() {
                    None
                } else {
                    Some(v.build.as_str().to_string())
                },
            }
        } else {
            ParsedSemver {
                original: info.original.clone(),
                is_valid: false,
                major: None,
                minor: None,
                patch: None,
                pre: None,
                build: None,
            }
        }
    }
}

impl std::cmp::PartialOrd for VersionInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for VersionInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.version.as_ref(), other.version.as_ref()) {
            (Some(x), Some(y)) => x.cmp(y),
            (Some(_), _) => std::cmp::Ordering::Greater,
            (_, Some(_)) => std::cmp::Ordering::Less,
            _ => self.original.cmp(&other.original),
        }
    }
}

impl std::fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(x) = &self.version {
            let j = json!({
                "original": self.original,
                "major": x.major,
                "minor": x.minor,
                "patch": x.patch,
                "pre": x.pre.as_str(),
                "build": x.build.as_str(),
            })
            .to_string();
            write!(f, "{}", j)
        } else {
            let j = json!({
                "original": self.original,
            })
            .to_string();
            write!(f, "{}", j)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_parse {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = VersionInfo::parse($input);
                assert_eq!($want, got);
            }
        };
    }

    test_parse!(
        test_parse_failure,
        "invalid",
        VersionInfo {
            original: "invalid".to_string(),
            version: None,
        }
    );
    test_parse!(
        test_parse_success,
        "1.2.3",
        VersionInfo {
            original: "1.2.3".to_string(),
            version: Some(Version::new(1, 2, 3)),
        }
    );
    test_parse!(
        test_parse_success_with_v,
        "v1.2.3",
        VersionInfo {
            original: "v1.2.3".to_string(),
            version: Some(Version::new(1, 2, 3)),
        }
    );

    #[test]
    fn test_parsed_semver_from_valid() {
        let info = VersionInfo::parse("v1.2.3-alpha.1+dev");
        let parsed = ParsedSemver::from(&info);
        assert!(parsed.is_valid);
        assert_eq!(parsed.original, "v1.2.3-alpha.1+dev");
        assert_eq!(parsed.major, Some(1));
        assert_eq!(parsed.minor, Some(2));
        assert_eq!(parsed.patch, Some(3));
        assert_eq!(parsed.pre, Some("alpha.1".to_string()));
        assert_eq!(parsed.build, Some("dev".to_string()));
    }

    #[test]
    fn test_parsed_semver_from_invalid() {
        let info = VersionInfo::parse("not-semver");
        let parsed = ParsedSemver::from(&info);
        assert!(!parsed.is_valid);
        assert_eq!(parsed.original, "not-semver");
        assert_eq!(parsed.major, None);
    }
}
