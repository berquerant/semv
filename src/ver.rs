use semver::Version;
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
}
