# papa
Papa Scorch's cli mod manager for Northstar clients and servers

## Usage

```bash
#must be as the URL appears on thunderstore e.g. /Fifty/Server_Utilities/ 
papa install Server_Utilities #install a mod
papa list #list installed mods
papa remove Server_Utilitiess #uninstall a mod
papa clear #clear the download cache
```

## Installation
Regardless of which method you use, I recommend setting your mods directory to something useful before using `papa`
```bash
papa config -m /PATH/TO/MODS/FOLDER/
```

### Ubuntu/Debian(& derivatives)
Download the `.deb` file below and install it using whatever you usually use to install packages:
```bash
sudo apt install ./papa_1.0.0.rc.1.deb
```

### Windows
Download and run the `.msi` installer below. It *should* install the program into your PATH so that you can call it from anywhere on your machine. This also makes for easy updates or uninstalls down the road.

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
