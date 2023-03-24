<h1 align="center">
 <!-- Formatting idea shamelessly stolen from https://github.com/OneGal/viper ty for the idea :) -->
 <img src="https://static.wikia.nocookie.net/titanfall/images/d/d5/ScorchIcon.png" />
 <br>
 Papa
<br>
<a href="https://github.com/AnActualEmerald/papa/actions/workflows/rust.yml"> 
 <img alt="Rust workflow badge" src="https://github.com/AnActualEmerald/papa/actions/workflows/rust.yml/badge.svg">
</a>
<img alt="Crates.io (latest)" src="https://img.shields.io/crates/dv/papa">
</h1>


<p align="center">Mod manager CLI for <a href="https://github.com/R2Northstar/Northstar">Northstar</a></p>

## Features
- Install and update Northstar from the command line
- Search Thunderstore for mods from the command line
- Download a mod *and* its dependencies with one command
- Easily keep your mods up to date

## Usage

```bash
papa install fifty.server_utilities #install a mod
papa list #list installed mods
papa update #update any out of date mods
papa remove fifty.server_utilities #uninstall a mod
```

## Installation
I suggest that you initialize Northstar to set everything up automatically
```bash
papa ns init
```
Or create a file at `.config/papa/config.toml` and set `install_dir` to whatever directory you want

### Ubuntu/Debian(& derivatives)
Download the `.deb` file from the latest release and install it using whatever you usually use to install packages:
```bash

sudo apt install ./papa_3.0.0.deb

```

### Arch Linux
Community maintained `papa` and `papa-bin` packages are available on the AUR:
```bash
paru -S papa
```

### Windows
Download and run the `.msi` installer from the latest release.

### Using prebuilt binaries
Download the appropriate binary for your system (make sure you get the `.exe` for Windows) and place it somewhere in your PATH. You should then be able to call the `papa` command from your favorite command line.

### Building from source
If you have cargo installed on your system, you should be able to install `papa` directly from [crates.io](https://crates.io)
```bash
 cargo install papa
```
or from the git repo
```bash
 cargo install --git https://github.com/AnActualEmerald/papa
```
If you want to build from source but don't have cargo installed, you should check out [rustup.rs](https://rustup.rs)
#### Dependencies
* pkgconfig
* openssl

## Caveats 
- The default install directory is **relative to the current working directory**, meaning that running `papa install` in `~/` will install mods into `~/mods`
