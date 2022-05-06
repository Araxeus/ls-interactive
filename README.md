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

![image](https://user-images.githubusercontent.com/78568641/167173566-8762a3a8-4dbf-492a-9883-f48760637bcd.png)

## Features

🌟 Navigate between folders using arrow keys + enter button

🌟 Open files with native apps using <kbd>Enter</kbd>

🌟 Top button (📁 ..) opens the parent directory

🌟 Bottom button (💻) opens current directory in your native file explorer

🌟 Press <kbd>Esc</kbd> to exit


## Installation

Download binaries from [releases page](https://github.com/Araxeus/ls-interactive/releases) and place in PATH

Installation from package managers is coming soon

## How to run it

```bash
lsi
```
or
```bash
lsi some_directory
```

## Build it yourself
(The binaries in release are [automatically](https://github.com/Araxeus/ls-interactive/blob/master/.github/workflows/release.yml) built by github actions)

install `rust`
clone/download the repo

run in project directory:
* `cargo run`: to run in dev mode
* `cargo build --release`: to build locally, output will be in `target\release` and named named `lsi`
