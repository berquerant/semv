use crate::ver::VersionInfo;
use semver::VersionReq;
use std::io::{self, BufRead};

type StringIter = dyn Iterator<Item = String>;
type StaticStringIter = dyn Iterator<Item = String> + 'static;
type VersionInfoIter = dyn Iterator<Item = VersionInfo>;
type StaticVersionInfoIter = dyn Iterator<Item = VersionInfo> + 'static;

pub fn read_lines(targets: Vec<String>) -> Box<StringIter> {
    if targets.is_empty() {
        let stdin = io::stdin();
        Box::new(stdin.lock().lines().map_while(Result::ok))
    } else {
        Box::new(targets.into_iter())
    }
}

pub fn print_lines(source: Box<StaticStringIter>) {
    for x in source {
        println!("{}", x);
    }
}

pub fn parse_versions(source: Box<StaticStringIter>) -> Box<VersionInfoIter> {
    Box::new(source.map(|x| VersionInfo::parse(&x)))
}

pub fn filter_semver(source: Box<StaticVersionInfoIter>, invert: bool) -> Box<VersionInfoIter> {
    Box::new(source.filter(move |x| invert != x.version.is_some()))
}

fn simple_output(source: Box<StaticVersionInfoIter>) -> Box<StringIter> {
    Box::new(source.map(|x| x.original))
}

fn verbose_output(source: Box<StaticVersionInfoIter>) -> Box<StringIter> {
    Box::new(source.map(|x| x.to_string()))
}

pub fn format_output(source: Box<StaticVersionInfoIter>, verbose: bool) -> Box<StringIter> {
    if verbose {
        verbose_output(source)
    } else {
        simple_output(source)
    }
}

pub fn filter_by_requirement(
    source: Box<StaticVersionInfoIter>,
    req: VersionReq,
) -> Box<VersionInfoIter> {
    Box::new(source.filter(move |x| {
        if let Some(v) = &x.version {
            req.matches(v)
        } else {
            true
        }
    }))
}

pub fn sort_lines(source: Box<StaticVersionInfoIter>, reverse: bool) -> Box<VersionInfoIter> {
    let mut elems: Vec<_> = source.collect();
    if reverse {
        elems.sort_by(|a, b| b.cmp(a));
    } else {
        elems.sort();
    }
    Box::new(elems.into_iter())
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_filter_semver {
        ($name:ident, $input:expr, $invert:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got_iter = filter_semver(Box::new($input.into_iter()), $invert);
                let got: Vec<VersionInfo> = got_iter.collect();
                assert_eq!($want, got);
            }
        };
    }

    test_filter_semver!(
        test_filter_semver_empty,
        Vec::new(),
        false,
        Vec::<VersionInfo>::new()
    );
    test_filter_semver!(
        test_filter_semver_semver_one,
        vec![VersionInfo::parse("invalid")],
        false,
        Vec::<VersionInfo>::new()
    );
    test_filter_semver!(
        test_filter_semver_ignore_non_semver,
        vec![VersionInfo::parse("invalid"), VersionInfo::parse("1.2.3")],
        false,
        vec![VersionInfo::parse("1.2.3")]
    );
    test_filter_semver!(
        test_filter_semver_ignore_semver,
        vec![VersionInfo::parse("invalid"), VersionInfo::parse("1.2.3")],
        true,
        vec![VersionInfo::parse("invalid")]
    );

    macro_rules! test_sort_lines {
        ($name:ident, $input:expr, $reverse:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got_iter = sort_lines(Box::new($input.into_iter()), $reverse);
                let got: Vec<VersionInfo> = got_iter.collect();
                assert_eq!($want, got);
            }
        };
    }

    test_sort_lines!(
        test_sort_lines_empty,
        Vec::new(),
        false,
        Vec::<VersionInfo>::new()
    );
    test_sort_lines!(
        test_sort_lines_one,
        vec![VersionInfo::parse("1.2.3")],
        false,
        vec![VersionInfo::parse("1.2.3")]
    );
    test_sort_lines!(
        test_sort_lines_sort,
        vec![
            VersionInfo::parse("1.2.3"),
            VersionInfo::parse("0.1.2"),
            VersionInfo::parse("1.2.4")
        ],
        false,
        vec![
            VersionInfo::parse("0.1.2"),
            VersionInfo::parse("1.2.3"),
            VersionInfo::parse("1.2.4")
        ]
    );
    test_sort_lines!(
        test_sort_lines_sort_reverse,
        vec![
            VersionInfo::parse("1.2.3"),
            VersionInfo::parse("0.1.2"),
            VersionInfo::parse("1.2.4")
        ],
        true,
        vec![
            VersionInfo::parse("1.2.4"),
            VersionInfo::parse("1.2.3"),
            VersionInfo::parse("0.1.2")
        ]
    );

    macro_rules! test_filter_by_requirement {
        ($name:ident, $source:expr, $req:expr, $want:expr) => {
            #[test]
            fn $name() {
                let got_iter = filter_by_requirement(Box::new($source.into_iter()), $req);
                let got: Vec<VersionInfo> = got_iter.collect();
                assert_eq!($want, got);
            }
        };
    }

    test_filter_by_requirement!(
        test_filter_by_requirement_empty,
        Vec::new(),
        VersionReq::parse(">=1.2.3, <1.8.0").unwrap(),
        Vec::<VersionInfo>::new()
    );
    test_filter_by_requirement!(
        test_filter_by_requirement_pass,
        vec![VersionInfo::parse("1.2.3")],
        VersionReq::parse(">=1.2.3, <1.8.0").unwrap(),
        vec![VersionInfo::parse("1.2.3")]
    );
    test_filter_by_requirement!(
        test_filter_by_requirement_partial_deny,
        vec![VersionInfo::parse("1.2.2"), VersionInfo::parse("1.2.3")],
        VersionReq::parse(">=1.2.3, <1.8.0").unwrap(),
        vec![VersionInfo::parse("1.2.3")]
    );
}
