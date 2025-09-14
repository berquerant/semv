# semv

```
❯ semv --help
semver inspection utilities

Usage: semv [OPTIONS] [TARGET]...

Arguments:
  [TARGET]...
          Specify the semver strings. If not specified, the semver strings will be read from stdin

Options:
  -s, --sort
          Sort results

  -v, --verbose
          Verbose output

  -r, --requirement <VERSION_REQUIREMENT>
          SemVer version requirement to filter results

  -z, --nonsemver
          Display non-semver strings only

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

Examples:
// Filter semver strings
> semver a 0.1.2 b 1.2.3-alpha.1
0.1.2
1.2.3-alpha.1
// Filter non-semver strings
> semv -z <<EOS
a
0.1.2
b
1.2.3-alpha.1
EOS
a
b
// Sort semver strings
> semv -s <<EOS
1.2.3
1.2.2
2.0.0
EOS
1.2.2
1.2.3
2.0.0
// Sort tags as semver strings
> semv -s <<EOS
v1.2.3
v1.2.2
v2.0.0
EOS
1.2.2
1.2.3
2.0.0
// Filter by requirement
> semv -r '>=1.2.0' <<EOS
1.1.0
1.3.0
2.0.0
EOS
1.3.0
2.0.0
// Verbose output
> semv -v <<EOS
1.1.0
1.3.0-alpha.1
2.0.0-alpha.1+dev
EOS
{"build":"","major":1,"minor":1,"original":"1.1.0","patch":0,"pre":""}
{"build":"","major":1,"minor":3,"original":"1.3.0-alpha.1","patch":0,"pre":"alpha.1"}
{"build":"dev","major":2,"minor":0,"original":"2.0.0-alpha.1+dev","patch":0,"pre":"alpha.1"}
// Filter by requirement, sort, verbose output
> semv -r '>1.2.0' -s -v <<EOS
3.0.0
1.0.0
2.1.3
1.5.0
EOS
{"build":"","major":1,"minor":5,"original":"1.5.0","patch":0,"pre":""}
{"build":"","major":2,"minor":1,"original":"2.1.3","patch":3,"pre":""}
{"build":"","major":3,"minor":0,"original":"3.0.0","patch":0,"pre":""}
```
