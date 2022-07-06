# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Global install option (unix only)
- `inlcude` and `exclude` commands for using global mods (unix only)
- Options to `list` for global and all mods
- `update` now lists the mods it will update, similar to `install`

### Changed
- Improved formatting a bit when installing many mods

### Fixed
- Cache breaking for packages with `_` in the package name
- Cache not actually clearing without `force` option



## v2.1.1

### Added 
- `force` option to `install` command
- Cache now cleans older versions of packages

### Fixed
- Running `search` with no terms returning no results rather than everything
- `install` asking to install nothing



## v2.1.0

### Added
- Northstar `install`, `init`, and `update` commands
- File overwrite warning on `update`
- Overwrite protection for `.cfg` files
- debug flag for base command

### Chagned
- Internal error handling
- Slimmed binary sized a bit 

### Fixed
- `update` not respecting disabled status
- `disable` and `enable` not properly modifying all sub mods if the parent mod's name was used
- `install` potentially using an outdated version from the cache


## v2.0.0

### Added
- Northstar `install`, `init`, and `update` commands
- File overwrite warning on `update`
- Overwrite protection for `.cfg` files
- debug flag for base command

### Chagned
- Internal error handling
- Slimmed binary sized a bit 

### Fixed
- `update` not respecting disabled status
- `disable` and `enable` not properly modifying all sub mods if the parent mod's name was used
- `install` potentially using an outdated version from the cache


## v2.0.0

### Added
- `search` command
- Support "bundle" type mods
- `enable` and `disable` commands

### Changed
- Improved list formatting
- More colorful output for readability


## v1.0.0

### Added 
- `update` command
- Installed mod tracking
- Support ~~somewhat~~ non-standard directory structures
- Pre-installation confirmation

### Changed
- Get download URLs from thunderstore api
- Use only package name for selecting packages
- `install` and `remove` are now case-insensitive
- Formatting changes



## v0.2.0

### Added
- `config` command to manage configuration

### Changed
- Improved `list` command formatting

### Fixed
- `remove` command not working on windows
- Install message not always getting package name

