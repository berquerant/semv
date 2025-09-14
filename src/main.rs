mod cli;
mod proc;
mod ver;

use crate::cli::Cli;
use crate::proc::{
    filter_by_requirement, filter_semver, print_lines, read_lines, sort_lines, verbose_output,
};
use clap::Parser;

fn main() {
    let opt = Cli::parse();
    let mut source = read_lines(opt.targets);
    source = filter_semver(source, opt.filter_non_semver);

    if !opt.filter_non_semver {
        if let Some(req) = opt.version_requirement {
            source = filter_by_requirement(source, req);
        }
        if opt.sort {
            source = sort_lines(source);
        }
        if opt.verbose {
            source = verbose_output(source);
        }
    }

    print_lines(source);
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;

    fn bin() -> assert_cmd::Command {
        Command::cargo_bin("semv").unwrap()
    }

    #[test]
    fn test_help() {
        bin().arg("--help").assert().success();
    }

    #[test]
    fn test_filter_semver_args() {
        bin()
            .args(&["a", "0.1.2", "b", "1.2.3-alpha.1"])
            .assert()
            .success()
            .stdout(predicates::str::diff(
                r#"0.1.2
1.2.3-alpha.1
"#,
            ));
    }

    #[test]
    fn test_filter_non_semver_stdin() {
        bin()
            .arg("--nonsemver")
            .write_stdin(
                r#"a
0.1.2
b
1.2.3-alpha.1"#,
            )
            .assert()
            .success()
            .stdout(predicates::str::diff(
                r#"a
b
"#,
            ));
    }

    #[test]
    fn test_filter_semver_tag_args() {
        bin()
            .args(&["a", "v0.1.2", "b", "1.2.3-alpha.1"])
            .assert()
            .success()
            .stdout(predicates::str::diff(
                r#"0.1.2
1.2.3-alpha.1
"#,
            ));
    }

    #[test]
    fn test_sort() {
        bin()
            .arg("--sort")
            .write_stdin(
                r#"1.2.3
1.2.2
2.0.0"#,
            )
            .assert()
            .success()
            .stdout(predicates::str::diff(
                r#"1.2.2
1.2.3
2.0.0
"#,
            ));
    }

    #[test]
    fn test_sort_tags() {
        bin()
            .arg("--sort")
            .write_stdin(
                r#"v1.2.3
v1.2.2
v2.0.0"#,
            )
            .assert()
            .success()
            .stdout(predicates::str::diff(
                r#"1.2.2
1.2.3
2.0.0
"#,
            ));
    }

    #[test]
    fn test_requirement() {
        bin()
            .arg("-r")
            .arg(">=1.2.0")
            .write_stdin(
                r#"1.1.0
1.3.0
2.0.0"#,
            )
            .assert()
            .success()
            .stdout(predicates::str::diff(
                r#"1.3.0
2.0.0
"#,
            ));
    }

    #[test]
    fn test_verbose() {
        bin()
            .arg("-v")
            .write_stdin(
                r#"1.1.0
1.3.0-alpha.1
2.0.0-alpha.1+dev"#,
            )
            .assert()
            .success()
            .stdout(predicates::str::diff(
                r#"{"build":"","major":1,"minor":1,"original":"1.1.0","patch":0,"pre":""}
{"build":"","major":1,"minor":3,"original":"1.3.0-alpha.1","patch":0,"pre":"alpha.1"}
{"build":"dev","major":2,"minor":0,"original":"2.0.0-alpha.1+dev","patch":0,"pre":"alpha.1"}
"#,
            ));
    }

    #[test]
    fn test_requirement_sort_verbose() {
        bin()
            .arg("-r")
            .arg(">1.2.0")
            .arg("-s")
            .arg("-v")
            .write_stdin(
                r#"3.0.0
1.0.0
2.1.3
1.5.0"#,
            )
            .assert()
            .success()
            .stdout(predicates::str::diff(
                r#"{"build":"","major":1,"minor":5,"original":"1.5.0","patch":0,"pre":""}
{"build":"","major":2,"minor":1,"original":"2.1.3","patch":3,"pre":""}
{"build":"","major":3,"minor":0,"original":"3.0.0","patch":0,"pre":""}
"#,
            ));
    }
}
