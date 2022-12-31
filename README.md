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

## ⚡ Features

🌟 Navigate between folders using arrow keys

🌟 Browse folders using <kbd>Enter</kbd>

🌟 Open folder in terminal (cd to folder) using [<kbd>Shift</kbd> + <kbd>Enter</kbd>] or [<kbd>Alt</kbd> + <kbd>Enter</kbd>]

🌟 Open folder in file manager using <kbd>Ctrl</kbd> + <kbd>Enter</kbd>

🌟 Open files with native apps using <kbd>Enter</kbd>

🌟 Top button (📁 ..) opens the parent directory

🌟 Press <kbd>Esc</kbd> to exit

🌟 Type anything to filter current folder content using fuzzy search

> on Linux/Mac <kbd>Shift</kbd> + <kbd>Enter</kbd> or <kbd>Ctrl</kbd> + <kbd>Enter</kbd> might not work
>
> see https://github.com/crossterm-rs/crossterm/issues/669

## 🛠 Installation

1. Download zip package from [releases page](https://github.com/Araxeus/ls-interactive/releases)
   
2. Extract its content into a folder in PATH ([guide](https://gist.github.com/nex3/c395b2f8fd4b02068be37c961301caa7))
   
3. Follow shell specific instructions:
<details>
  <summary><bold>Bash (Linux/Mac)</bold></summary>

  * Copy the `lsi` function from [scripts/lsi.sh](https://github.com/Araxeus/ls-interactive/blob/master/scripts/lsi.sh) into to `/home/user/.bashrc`

    ```bash
    gedit /home/user/.bashrc
    ```
 
</details>

<details>
  <summary><bold>Batch (Windows)</bold></summary>

  * Copy [scripts/lsi.bat](https://github.com/Araxeus/ls-interactive/blob/master/scripts/lsi.sh) into a folder in your `%PATH%` environment variable

    you can open you envionment variables settings using the following command:

    ```batch
    rundll32.exe sysdm.cpl,EditEnvironmentVariables
    ```
 
</details>

<details>
  <summary><bold>Fish Shell</bold></summary>

  * Copy [scripts/lsi.fish](https://github.com/Araxeus/ls-interactive/blob/master/scripts/lsi.fish) into `~/.config/fish/functions/`
   
    (Windows WSL is currently unsupported)

  
 
</details>
<details>
  <summary><bold>Powershell</bold></summary>

  * Copy the `lsi` function from [scripts/lsi.ps1](https://github.com/Araxeus/ls-interactive/blob/master/scripts/lsi.ps1) to your `Microsoft.PowerShell_profile.ps1`

    you can open your profile using one of the following commands:

    ```ps1
    notepad $profile
    ```
    <br>
   
    ```ps1
    gedit $profile
    ```
 
</details>

<details>
  <summary><bold>Nushell</bold></summary>

  * Copy the `lsi` function from [scripts/lsi.nu](https://github.com/Araxeus/ls-interactive/blob/master/scripts/lsi.nu) to your `env.nu`

    you can open your profile using one of the following commands:

    ```bash
    notepad $nu.env-path
    ```
    <br>
   
    ```bash
    gedit $nu.env-path
    ```
 
</details>

## 💻 How to run it

```bash
lsi
```

or

```bash
lsi some_directory
```

## ⚙️ Build it yourself

(Releases are [automatically](https://github.com/Araxeus/ls-interactive/blob/master/.github/workflows/release.yml) built by github actions)

-   install `rust`
-   clone/download the repo

-   run in project directory:
    -   `cargo run`: to run in dev mode
    -   `cargo build --release`: to build locally,
        -   executable will be in `/target/release` and named `ls_interactive`
        -   launch script will be in the `/scripts` directory

## 🤝Contributing

Contributions, issues and feature requests are welcome!<br />Feel free to check [issues page](https://github.com/Araxeus/ls-interactive/issues).

Before submitting a Pull Request, verify your changes with all following commands:

```mcfunction
cargo check
```

```mcfunction
cargo fmt --all --check
```

```mcfunction
cargo clippy --all-targets --all-features -- -W clippy::pedantic -W clippy::cargo -W clippy::nursery
```

## ❤️ Show your support

Give a ⭐ if this package helped you!

## 📜 License

MIT. See [LICENSE file](./LICENSE) for details.
