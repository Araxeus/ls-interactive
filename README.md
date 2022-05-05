<h1 align="center">ls-interactive 💻</h1>
<p>
  <a href="https://github.com/Araxeus/ls-interactive/releases" target="_blank">
    <img alt="Version" src="https://img.shields.io/github/release/Araxeus/ls-interactive.svg" onerror='this.onerror=undefined; this.src="https://img.shields.io/badge/version-1.0.0-blue.svg?cacheSeconds=2592000"'/>
  </a>
  <a href="https://github.com/Araxeus/ls-interactive/blob/main/LICENSE" target="_blank">
    <img alt="License: MIT" src="https://img.shields.io/github/license/Araxeus/ls-interactive?color=yellow" />
  </a>
   <a href="https://github.com/Araxeus/ls-interactive" target="_blank">
    <img alt="Maintenance" src="https://img.shields.io/badge/Maintained%3F-yes-green.svg" />
  </a>
</p>

> Interactive ls command made in rust


## Installation

Download binaries from [releases page](https://github.com/Araxeus/ls-interactive/releases) and place in PATH

or run:
```bash
curl -LSfs https://japaric.github.io/trust/install.sh | sh -s -- --git Araxeus/ls-interactive
```

## How to run it

```bash
lsi
```
or
```bash
lsi some_directory
```

## Build it yourself
(The binaries in release are [automatically](https://github.com/Araxeus/ls-interactive/blob/main/.github/workflows/release.yml) built by github actions)

install `rust`
clone/download the repo

run in project directory:
* `cargo run`: to run in dev mode
* `cargo build --release`: to build locally, output will be in `target\release` and named named `lsi`
