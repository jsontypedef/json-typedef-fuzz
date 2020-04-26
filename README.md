# jtd-fuzz

`jtd-fuzz` generates example data from a JSON Typedef schema.

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

### Install with Homebrew

This option is recommended if you're on macOS.

```bash
brew install jsontypedef/jsontypedef/jtd-fuzz
```

### Install with Docker

This option is recommended on non-Mac platforms, or if you're running `jtd-fuzz`
in some sort of script and you want to make sure that everyone running the
script uses the same version of `jtd-fuzz`.

```bash
docker pull jsontypedef/jtd-tools
```

If you opt to use the Docker approach, you will need to change all invocations
of `jtd-fuzz` in this README from:

```bash
jtd-fuzz [...]
```

To:

```bash
docker exec -it jsontypedef/jtd-tools /jtd-fuzz [...]
```

### Install with Cargo

This option is recommended if you already have `cargo` installed, or if you
would prefer to use a version of `jtd-fuzz` compiled on your machine:

```bash
cargo install jtd-fuzz
```

## Usage

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

## Advanced Usage: Providing a Seed

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
echo '{}' | cargo run -- -n 1 -s 1815 ; echo '{}' | cargo run -- -n 1 -s 1815
```

```json
{"":null,"8= Rk":0.4883371274545145,";":0.6198663347278088,"Zg":[["O^;BZ",92,null,["9]0MZ_q",null,{}]],null,0.7705810550086981,"xb",false,"i","h"],"m83":115}
{"":null,"8= Rk":0.4883371274545145,";":0.6198663347278088,"Zg":[["O^;BZ",92,null,["9]0MZ_q",null,{}]],null,0.7705810550086981,"xb",false,"i","h"],"m83":115}
```

The `-s` option takes an integer between 0 and 2^64 - 1.

The `-s` option can be useful if you're using `jtd-fuzz` to do automated testing
against a system. Your automated testing system can pass `jtd-fuzz` a
randomly-generated seed, and if the automated tester finds a seed that generates
problematic output, it can output the seed it used. That way, developers can
re-use that seed, and try to reproduce the issue locally.

Note that `jtd-fuzz` is only guaranteed to produce consistent output if you use
the same seed, schema, and version of `jtd-fuzz`. Different versions on
`jtd-fuzz` may output different results, even if you give them the same seed and
schema.
