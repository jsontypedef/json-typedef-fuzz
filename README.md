# jtd-fuzz

`jtd-fuzz` generates example data [(aka "fuzz tests")][fuzz] from a JSON Typedef
schema.

```bash
echo '{ "elements": { "type": "string" }}' | jtd-fuzz -n 5
```

```json
["_","/+Z`","8o~5[7A"]
[]
["@(;","*+!YVz"]
["u4sv>Sp","Uc","o`"]
["","G","*ZJsc","","","\"RT,","l>l"]
```

A note on security: `jtd-fuzz` does not use a cryptographically-secure
random-number generator. Do not use `jtd-fuzz` to generate randomness, if that
randomness needs to be cryptographically secure.

## Installation

To install `jtd-fuzz`, you have a few options:

### Install on macOS

You can install `jtd-fuzz` via Homebrew:

```bash
brew install jsontypedef/jsontypedef/jtd-fuzz
```

Alternatively, you can download and extract the binary yourself from
`x86_64-apple-darwin.zip` in [the latest release][latest]. Because of Apple's
quarantine system, you will either need to run:

```bash
xattr -d com.apple.quarantine path/to/jtd-fuzz
```

In order to be able to run the executable.

### Install on Linux

Download and extract the binary from `x86_64-unknown-linux-gnu.zip` in [the
latest release][latest].

### Install on Windows

Download and extract the binary from `x86_64-pc-windows-gnu.zip` in [the latest
release][latest]. Runs on 64-bit MinGW for Windows 7+.

### Install with Docker

This option is recommended if you're running `jtd-fuzz` in some sort of script
and you want to make sure that everyone running the script uses the same version
of `jtd-fuzz`.

```bash
docker pull jsontypedef/jtd-fuzz
```

If you opt to use the Docker approach, you will need to change all invocations
of `jtd-fuzz` in this README from:

```bash
jtd-fuzz [...]
```

To:

```bash
# To have jtd-fuzz read from STDIN, run it like so:
docker exec -i jsontypedef/jtd-fuzz [...]

# To have jtd-fuzz read from a file, run it as:
docker run -v /path/to/file.json:/file.json -i jsontypedef/jtd-fuzz [...] file.json
# or, if file.json is in your current directory:
docker run -v $(pwd)/file.json:/file.json -i jsontypedef/jtd-fuzz [...] file.json
```

## Usage

### Basic Usage

To invoke `jtd-fuzz`, you can either:

1. Have it read from STDIN. This is the default behavior.
2. Have it read from a file. To do this, pass a file name as the last argument
   to `jtd-fuzz`.

`jtd-fuzz` reads in a single JSON Typedef schema, and will output, by default,
an infinite stream of examples. For example, this will output an infinite
sequence of random JSON data:

```bash
echo '{}' | jtd-fuzz
```

If you'd like to have `jtd-fuzz` output an exact number of results, use `-n`:

```bash
echo '{}' | jtd-fuzz -n 5
```

Or, to have `jtd-fuzz` read from a file:

```bash
echo '{ "type": "timestamp" }' > foo.jtd.json

jtd-fuzz -n 5 foo.jtd.json
```

### Advanced Usage: Providing a Seed

By default, `jtd-fuzz` will generate different output every time:

```bash
echo '{}' | jtd-fuzz -n 1 ; echo '{}' | jtd-fuzz -n 1
```

```json
{"[jD|6W":null}
null
```

If you'd like to get consistent output from `jtd-fuzz`, or be able to reproduce
its output, you can use the `-s` option to provide a seed to its internal
pseudo-random number generator. For the same seed and schema, `jtd-fuzz` will
output the same data every time:

```bash
echo '{}' | jtd-fuzz -n 1 -s 8927 ; echo '{}' | jtd-fuzz -n 1 -s 8927
```

```json
48
48
```

The `-s` option takes an integer between 0 and 2^64 - 1.

Seeding `jtd-fuzz` can be useful if you're using `jtd-fuzz` to do automated
testing against a system. Your automated testing system can pass `jtd-fuzz` a
randomly-generated seed, and if the automated tester finds a seed that reveals a
bug, it can output the seed it used. That way, developers can re-use that seed,
and try to reproduce the issue locally.

Note that `jtd-fuzz` is only guaranteed to produce consistent output if you use
the same seed, schema, and version of `jtd-fuzz`. Different versions on
`jtd-fuzz` may output different results, even if you give them the same seed and
schema.

[fuzz]: https://en.wikipedia.org/wiki/Fuzzing
[latest]: https://github.com/jsontypedef/json-typedef-fuzz/releases/latest
