<h1 align="center">
 <!-- Formatting idea shamelessly stolen from https://github.com/OneGal/viper ty for the idea :) -->
 <img src="https://static.wikia.nocookie.net/titanfall/images/d/d5/ScorchIcon.png" />
 <br>
 Papa
<br>
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/AnActualEmerald/papa/Rust) ![Crates.io (latest)](https://img.shields.io/crates/dv/papa)
</h1>


<p align="center">Mod manager CLI for <a href="https://github.com/R2Northstar/Northstar">Northstar</a></p>

## Features
- Install and update Northstar from the command line
- Search Thunderstore for mods from the command line
- Download a mod *and* its dependencies with one command
- Easily keep your mods up to date
- Per-directory tracking makes hosting multiple servers with different mods from one machine easy
- Enable and disable mods independent of N*'s own enabling and disabling

## Usage

```bash
papa install server_utilities #install a mod
papa list #list installed mods
papa update #update any out of date mods
papa remove server_utilities #uninstall a mod
papa clear #clear the download cache
```

## Installation
Regardless of which method you use, I recommend setting your mods directory to something useful before using `papa`
```bash
papa config -m /PATH/TO/MODS/FOLDER/
```
Or initialize Northstar to set everything up automatically
```bash
papa ns init /PATH/TO/TITANFALL2/
```

### Ubuntu/Debian(& derivatives)
Download the `.deb` file from the latest release and install it using whatever you usually use to install packages:
```bash

sudo apt install ./papa_2.1.0.deb

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

## Caveats 
- The default install directory is **relative to the current working directory**, meaning that running the command in ~/ will install mods into ~/mods
- Installed mods are tracked by a `.papa.ron` file in the mods directory, so each directory will have its own list of mods
- For now updates will blow up any changes made within the mod files themselves
