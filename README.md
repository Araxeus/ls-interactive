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

🌟 Navigate between folders/files using arrow keys

🌟 Browse folders / Open files with native apps using <kbd>Enter</kbd>

🌟 Open folder in terminal (CD to folder) using <kbd>Shift</kbd>+<kbd>Enter</kbd> or <kbd>Alt</kbd>+<kbd>Enter</kbd>

🌟 Open folder in file manager using <kbd>Ctrl</kbd>+<kbd>Enter</kbd>

🌟 Type anything to filter current folder content using fuzzy search

🌟 Top button (📁 ..) opens the parent directory (<kbd>LeftArrow</kbd> can also be used when fuzzy text field is empty)

🌟 Press <kbd>Esc</kbd> to exit

> on Linux/Mac <kbd>Shift</kbd>+<kbd>Enter</kbd> or <kbd>Ctrl</kbd>+<kbd>Enter</kbd> *might* not work
>
> see https://github.com/crossterm-rs/crossterm/issues/669

## 🛠 Installation

1. Download zip package from [releases page](https://github.com/Araxeus/ls-interactive/releases)

2. Extract its content into a folder in PATH ([guide](https://gist.github.com/nex3/c395b2f8fd4b02068be37c961301caa7))

3. Choose variation and follow shell specific instructions:
<details>
<summary><bold>Open files with your chosen editor</bold></summary>

  <details>
    <summary><bold>Bash (Linux/Mac) or Zsh</bold></summary>

  * Copy the `lsi` function from [scripts/lsi.sh](./scripts/cli-editor/lsi.sh) to your `~/.bashrc` or `~/.zshrc` file:

      https://github.com/Araxeus/ls-interactive/blob/8104712d75b1a075802365ff6767ff1974d5eb3a/scripts/cli-editor/lsi.sh#L3-L8

  </details>

  <details>
    <summary><bold>Batch (Windows CMD)</bold></summary>

    * Copy [scripts/lsi.bat](./scripts/cli-editor/lsi.bat) into a folder that is in your `%PATH%` environment variable

      https://github.com/Araxeus/ls-interactive/blob/8104712d75b1a075802365ff6767ff1974d5eb3a/scripts/cli-editor/lsi.bat#L3-L5

      you can open you environment variables settings using the command below: (or by searching for `env` in the start menu)

      ```batch
      rundll32.exe sysdm.cpl,EditEnvironmentVariables
      ```

  </details>

  <details>
    <summary><bold>Fish Shell</bold></summary>

    * Copy [scripts/lsi.fish](./scripts/cli-editor/lsi.fish) into `~/.config/fish/functions/`

      OR copy the function inside into your `~/.config/fish/config.fish` file

      https://github.com/Araxeus/ls-interactive/blob/8104712d75b1a075802365ff6767ff1974d5eb3a/scripts/cli-editor/lsi.fish#L7-L13


  </details>
  <details>
    <summary><bold>Powershell</bold></summary>

    * Copy the `lsi` function from [scripts/lsi.ps1](./scripts/cli-editor/lsi.ps1) to your `Microsoft.PowerShell_profile.ps1`

      https://github.com/Araxeus/ls-interactive/blob/8104712d75b1a075802365ff6767ff1974d5eb3a/scripts/cli-editor/lsi.ps1#L7-L10

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

    * Copy the `lsi` function from [scripts/lsi.nu](./scripts/cli-editor/lsi.nu) to your `env.nu`

      https://github.com/Araxeus/ls-interactive/blob/8104712d75b1a075802365ff6767ff1974d5eb3a/scripts/cli-editor/lsi.nu#L4-L10

      you can open your environment file using the following command:

      ```bash
      config env
      ```

  </details>
</details>

<details>
<summary><bold>Open files with operating system</bold></summary>

  <details>
    <summary><bold>Bash (Linux/Mac) or Zsh</bold></summary>

  * Copy the `lsi` function from [scripts/lsi.sh](./scripts/lsi.sh) to your `~/.bashrc` or `~/.zshrc` file:

      https://github.com/Araxeus/ls-interactive/blob/f1cd2db8a7bddb5aee5e0d3e2482d85b11d76f31/scripts/lsi.sh#L3-L8

  </details>

  <details>
    <summary><bold>Batch (Windows CMD)</bold></summary>

    * Copy [scripts/lsi.bat](./scripts/lsi.bat) into a folder that is in your `%PATH%` environment variable

      https://github.com/Araxeus/ls-interactive/blob/f1cd2db8a7bddb5aee5e0d3e2482d85b11d76f31/scripts/lsi.bat#L3-L5

      you can open you environment variables settings using the command below: (or by searching for `env` in the start menu)

      ```batch
      rundll32.exe sysdm.cpl,EditEnvironmentVariables
      ```

  </details>

  <details>
    <summary><bold>Fish Shell</bold></summary>

    * Copy [scripts/lsi.fish](./scripts/lsi.fish) into `~/.config/fish/functions/`

      OR copy the function inside into your `~/.config/fish/config.fish` file

      https://github.com/Araxeus/ls-interactive/blob/f1cd2db8a7bddb5aee5e0d3e2482d85b11d76f31/scripts/lsi.fish#L7-L13


  </details>
  <details>
    <summary><bold>Powershell</bold></summary>

    * Copy the `lsi` function from [scripts/lsi.ps1](./scripts/lsi.ps1) to your `Microsoft.PowerShell_profile.ps1`

      https://github.com/Araxeus/ls-interactive/blob/f1cd2db8a7bddb5aee5e0d3e2482d85b11d76f31/scripts/lsi.ps1#L7-L10

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

    * Copy the `lsi` function from [scripts/lsi.nu](./scripts/lsi.nu) to your `env.nu`

      https://github.com/Araxeus/ls-interactive/blob/edab3551edc1a7e3a18b596166092b13eeffcf2f/scripts/lsi.nu#L4-L10

      you can open your environment file using the following command:

      ```bash
      config env
      ```

  </details>
</details>

## 💻 How to run it

```bash
lsi
```

or

```bash
lsi some_relative_or_absolute_path
```

## ⚙️ Build it yourself

(Releases are [automatically](./.github/workflows/release.yml) built by github actions)

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
