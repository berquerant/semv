use semver::Version;

fn trim_prefix_v(s: &str) -> &str {
    if let Some(x) = s.strip_prefix('v') {
        x
    } else {
        s
    }
}

pub fn parse(s: &str) -> Option<Version> {
    Version::parse(s)
        .or_else(|_| Version::parse(trim_prefix_v(s)))
        .ok()
}

pub fn is_semver(s: &str) -> bool {
    parse(s).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_parse {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got = parse($input);
                assert_eq!($want, got);
            }
        };
    }

    test_parse!(test_parse_failure, "invalid", None);
    test_parse!(test_parse_success, "1.2.3", Some(Version::new(1, 2, 3)));
    test_parse!(
        test_parse_success_with_v,
        "v1.2.3",
        Some(Version::new(1, 2, 3))
    );
}
