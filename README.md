# tracker

A terminal-based real-time satellite tracking and orbit prediction application.

<p align="center">
    <img src="docs/screenshot.png" width="80%" alt="Screenshot"><br/>
    <small><i>The font used in the screenshot is <a href="https://github.com/microsoft/cascadia-code">Cascadia Code NF</a>.</i></small>
</p>

## Features

- **Position and trajectory**: Displays current positions and trajectories using the SGP4 model.
- **Detailed information**: Provides comprehensive details about selected object.
- **Time adjustment**: View satellite positions at any past or future time.
- **Map scrolling**: Horizontal scrolling with infinite world map view.
- **Object following**: Follows selected object on the map.
- **Automatic updates**: Updates orbital elements automatically via internet.
- **Custom configuration**: Customizable settings for display and behavior.
- **Multi-language support**: Interface available in multiple languages.

## Installation

### Package manager

#### Arch Linux

`tracker` is available in the [AUR](https://aur.archlinux.org/packages/tracker/):

```bash
paru -S tracker # use your favorite AUR helper
```

#### Windows

```powershell
scoop bucket add extra
scoop install tracker
```

### Build from source

```bash
cargo install --git https://github.com/ShenMian/tracker
```

## Documentation

- [Keymap](docs/keymap.md).
- [Configuration](docs/configuration.md).

## License

Licensed under [Apache License, Version 2.0](LICENSE).
