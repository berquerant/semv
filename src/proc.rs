use crate::ver::{is_semver, parse};
use semver::{Version, VersionReq};
use serde_json::json;
use std::io::{self, BufRead};

type StringIter = dyn Iterator<Item = String>;
type StaticStringIter = dyn Iterator<Item = String> + 'static;
type VersionIter = dyn Iterator<Item = Version>;

pub fn print_lines(source: Box<StaticStringIter>) {
    for x in source {
        println!("{}", x);
    }
}

pub fn filter_semver(source: Box<StaticStringIter>, invert: bool) -> Box<StringIter> {
    if invert {
        Box::new(source.filter(|x| !is_semver(x)))
    } else {
        Box::new(parse_versions(source).map(|x| x.to_string()))
    }
}

fn parse_versions(source: Box<StaticStringIter>) -> Box<VersionIter> {
    let it = source.filter_map(|x| parse(&x));
    Box::new(it)
}

pub fn verbose_output(source: Box<StaticStringIter>) -> Box<StringIter> {
    let it = parse_versions(source).map(|x| {
        json!({
            "original": x.to_string(),
            "major": x.major,
            "minor": x.minor,
            "patch": x.patch,
            "pre": x.pre.as_str(),
            "build": x.build.as_str(),
        })
        .to_string()
    });
    Box::new(it)
}

pub fn filter_by_requirement(source: Box<StaticStringIter>, req: VersionReq) -> Box<StringIter> {
    let it = parse_versions(source)
        .filter(move |x| req.matches(x))
        .map(|x| x.to_string());
    Box::new(it)
}

pub fn sort_lines(source: Box<StaticStringIter>) -> Box<StringIter> {
    let it = parse_versions(source);
    let mut elems: Vec<_> = it.collect();
    elems.sort();
    let it = elems.into_iter().map(|x| x.to_string());
    Box::new(it)
}

pub fn read_lines(targets: Vec<String>) -> Box<StringIter> {
    if targets.is_empty() {
        let stdin = io::stdin();
        Box::new(stdin.lock().lines().map_while(Result::ok))
    } else {
        Box::new(targets.into_iter())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_parse_versions {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got_iter = parse_versions(Box::new($input.into_iter()));
                let got: Vec<Version> = got_iter.collect();
                assert_eq!($want, got);
            }
        };
    }

    test_parse_versions!(test_parse_versions_empty, Vec::new(), Vec::<Version>::new());
    test_parse_versions!(
        test_parse_versions_one,
        vec!["1.2.3".to_string()],
        vec![Version::new(1, 2, 3)]
    );
    test_parse_versions!(
        test_parse_versions_ignore_non_semver,
        vec!["1.2.3".to_string(), "invalid".to_string()],
        vec![Version::new(1, 2, 3)]
    );

    macro_rules! test_filter_semver {
        ($name:ident, $input:expr, $invert:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got_iter = filter_semver(Box::new($input.into_iter()), $invert);
                let got: Vec<String> = got_iter.collect();
                assert_eq!($want, got);
            }
        };
    }

    test_filter_semver!(
        test_filter_semver_empty,
        Vec::new(),
        false,
        Vec::<String>::new()
    );
    test_filter_semver!(
        test_filter_semver_semver_one,
        vec!["invalid".to_string()],
        false,
        Vec::<String>::new()
    );
    test_filter_semver!(
        test_filter_semver_ignore_non_semver,
        vec!["invalid".to_string(), "1.2.3".to_string()],
        false,
        vec!["1.2.3".to_string()]
    );
    test_filter_semver!(
        test_filter_semver_ignore_semver,
        vec!["invalid".to_string(), "1.2.3".to_string()],
        true,
        vec!["invalid".to_string()]
    );

    macro_rules! test_verbose_output {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got_iter = verbose_output(Box::new($input.into_iter()));
                let got: Vec<String> = got_iter.collect();
                assert_eq!($want, got);
            }
        };
    }

    test_verbose_output!(test_verbose_output_empty, Vec::new(), Vec::<String>::new());
    test_verbose_output!(
        test_verbose_output_one,
        vec!["1.2.3".to_string()],
        vec![r#"{"build":"","major":1,"minor":2,"original":"1.2.3","patch":3,"pre":""}"#]
    );
    test_verbose_output!(
        test_verbose_output_ignore_invalid_semver,
        vec!["1.2.3".to_string(), "invalid".to_string()],
        vec![r#"{"build":"","major":1,"minor":2,"original":"1.2.3","patch":3,"pre":""}"#]
    );

    macro_rules! test_sort_lines {
        ($name:ident, $input:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got_iter = sort_lines(Box::new($input.into_iter()));
                let got: Vec<String> = got_iter.collect();
                assert_eq!($want, got);
            }
        };
    }

    test_sort_lines!(test_sort_lines_empty, Vec::new(), Vec::<String>::new());
    test_sort_lines!(
        test_sort_lines_one,
        vec!["1.2.3".to_string()],
        vec!["1.2.3".to_string()]
    );
    test_sort_lines!(
        test_sort_lines_sort,
        vec![
            "1.2.3".to_string(),
            "0.1.2".to_string(),
            "1.2.4".to_string()
        ],
        vec![
            "0.1.2".to_string(),
            "1.2.3".to_string(),
            "1.2.4".to_string()
        ]
    );

    macro_rules! test_filter_by_requirement {
        ($name:ident, $source:expr, $req:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got_iter = filter_by_requirement(Box::new($source.into_iter()), $req);
                let got: Vec<String> = got_iter.collect();
                assert_eq!($want, got);
            }
        };
    }

    test_filter_by_requirement!(
        test_filter_by_requirement_empty,
        Vec::new(),
        VersionReq::parse(">=1.2.3, <1.8.0").unwrap(),
        Vec::<String>::new()
    );
    test_filter_by_requirement!(
        test_filter_by_requirement_pass,
        vec!["1.2.3".to_string()],
        VersionReq::parse(">=1.2.3, <1.8.0").unwrap(),
        vec!["1.2.3".to_string()]
    );
    test_filter_by_requirement!(
        test_filter_by_requirement_partial_deny,
        vec!["1.2.2".to_string(), "1.2.3".to_string()],
        VersionReq::parse(">=1.2.3, <1.8.0").unwrap(),
        vec!["1.2.3".to_string()]
    );
}
