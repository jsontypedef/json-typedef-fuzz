# jtd-fuzz [![Crates.io](https://img.shields.io/crates/v/jtd_fuzz)](https://crates.io/crates/jtd_fuzz) [![Docs.rs](https://docs.rs/jtd-fuzz/badge.svg)](https://docs.rs/jtd_fuzz)

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

## Installation

On macOS, you can install `jtd-fuzz` via Homebrew:

```bash
brew install jsontypedef/jsontypedef/jtd-fuzz
```

For all other platforms, you can download and extract the binary yourself from
[the latest release][latest]. You can also install using `cargo` by running:

```bash
cargo install jtd_fuzz
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

### Generating emails, names, etc. with `fuzzHint`

Oftentimes, it's useful for `jtd-fuzz` to generate specific sorts of strings,
instead of the generic nonsense strings you get by default with schemas whose
`type` is `string`. You can customize `jtd-fuzz`'s output using the `fuzzHint`
metadata property. For example, this schema:

```json
{
  "metadata": {
    "fuzzHint": "en_us/internet/email"
  },
  "type": "string"
}
```

Would, if you put it in a file called `example.jtd.json`, generate data like
this:

```bash
jtd-fuzz -n 5 example.jtd.json
```

```json
"nerdman9@bergnaum.name"
"christopkulas@crooks.biz"
"ykozey5@wiza.org"
"rowenakunde@lang.com"
"udouglas01@carter.info"
```

`fuzzHint` will only work on schemas of `{"type": "string"}`. Here are some
commonly-used values for `fuzzHint`:

- `en_us/company/company_name` generates strings like `Hayes, Murray, and Kiehn`
- `en_us/internet/email` generates strings like `alainatorphy@johnson.com`
- `en_us/names/full_name` generates strings like `Alexa Wisozk`

A full list of possible values for `fuzzHint` is available
[here](https://docs.rs/jtd-fuzz/0.2.0/jtd_fuzz/fn.fuzz.html#using-fuzzhint).

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

## Security Considerations

Do not rely on `jtd-fuzz` as a source of cryptographically secure randomness.
`jtd-fuzz` is meant to be used as a generator of example data, such as for fuzz
testing applications. It is not meant to be used for cryptographic purposes.

[fuzz]: https://en.wikipedia.org/wiki/Fuzzing
[latest]: https://github.com/jsontypedef/json-typedef-fuzz/releases/latest
