# tracker

A terminal-based real-time satellite tracking and orbit prediction application.

<p align="center">
    <img src="docs/screenshot.png" width="80%" alt="Screenshot"><br/>
    <small><i>The font used in the screenshot is <a href="https://github.com/microsoft/cascadia-code">Cascadia Code NF</a>.</i></small>
</p>

## Features

- **Orbit propagation**: Real-time positions & trajectories using SGP4.
- **Detailed info**: Object information.
- **Sky view**: Polar azimuth/elevation plot.
- **Time shift**: View past/future positions.
- **Object following**: Follow selected object.
- **Infinite map**: Continuous horizontal world map.
- **Auto updates**: Automatic TLE updates.
- **Configurable**: Custom display & behavior.
- **Localization**: UI translations.

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
