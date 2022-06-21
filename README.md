![scorch titan icon](https://static.wikia.nocookie.net/titanfall/images/d/d5/ScorchIcon.png)

# Papa command line mod manager
Mod manager cli for [Northstar](https://github.com/R2Northstar/Northstar)

## Features
- Search Thunderstore for mods from the command line
- Download a mod *and* its dependencies with one command
- Easily keep your mods up to date
- Per-directory tracking
- Enable and disable mods 

## Usage

```bash
papa install Server_Utilities #install a mod
papa list #list installed mods
papa update #update any out of date mods
papa remove Server_Utilities #uninstall a mod
papa clear #clear the download cache
```

## Installation
Regardless of which method you use, I recommend setting your mods directory to something useful before using `papa`
```bash
papa config -m /PATH/TO/MODS/FOLDER/
```

### Ubuntu/Debian(& derivatives)
Download the `.deb` file from the latest release and install it using whatever you usually use to install packages:
```bash
sudo apt install ./papa_2.0.0.deb
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
