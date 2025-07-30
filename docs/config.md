# Configuration

## Location

Default locations of configuration files on different platforms:

- Linux/macOS: `$HOME/.config/tracker/config.toml`.
- Windows: `%USERPROFILE%\.config\tracker\config.toml`.

## Default

```toml
[world_map]
follow_selected_object = false
lon_delta_deg = 10.0
time_delta_min = 1
map_color = "gray"
trajectory_color = "light_blue"
terminator_color = "dark_gray"

[satellite_groups]
cache_lifetime_min = 120
```
