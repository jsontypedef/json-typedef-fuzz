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
