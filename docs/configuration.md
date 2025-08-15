# Configuration

## Location

Default locations of configuration files on different platforms:

- Linux/macOS: `$HOME/.config/tracker/config.toml`.
- Windows: `%USERPROFILE%\.config\tracker\config.toml`.

## Default

```toml
[world_map]
follow_selected_object = true
show_terminator = true
show_cursor_position = false
lon_delta_deg = 10.0
time_delta_min = 1
map_color = "gray"
trajectory_color = "light_blue"
terminator_color = "dark_gray"

[satellite_groups]
cache_lifetime_min = 120
groups = [
    # Space Stations
	{ label = "ISS", cospar_id = "1998-067A" },
	{ label = "CSS", cospar_id = "2021-035A" },
    # Weather & Earth Resources Satellites
	{ label = "Weather", group_name = "weather" },
	{ label = "NOAA", group_name = "noaa" },
	{ label = "GOES", group_name = "goes" },
	{ label = "Earth resources", group_name = "resource" },
	{ label = "Search & rescue", group_name = "sarsat" },
	{ label = "Disaster monitoring", group_name = "dmc" },
    # Navigation Satellites
	{ label = "GPS Operational", group_name = "gps-ops" },
	{ label = "GLONASS Operational", group_name = "glo-ops" },
	{ label = "Galileo", group_name = "galileo" },
	{ label = "Beidou", group_name = "beidou" },
    # Scientific Satellites
	{ label = "Space & Earth Science", group_name = "science" },
	{ label = "Geodetic", group_name = "geodetic" },
	{ label = "Engineering", group_name = "engineering" },
	{ label = "Education", group_name = "education" },
    # Miscellaneous Satellites
	{ label = "Military", group_name = "military" },
	{ label = "Radar calibration", group_name = "radar" },
	{ label = "CubeSats", group_name = "cubesat" },
]
```

## Groups

Satellite TLE (Two-Line Element) data is retrieved from [CelesTrak](https://celestrak.org).

- **Grouped by function** – See [Current GP Element Sets](https://celestrak.org/NORAD/elements/) for available groups. The `group_name` field in the configuration corresponds to these group identifiers.
- **Individual satellites** – Use the [Search Satellite Catalog](https://celestrak.org/satcat/search.php) to find specific satellites. The `cospar_id` field corresponds to the satellite's International Designator.

## Color

Available colors:

- `black`.
- `red`.
- `green`.
- `yellow`.
- `blue`.
- `magenta`.
- `cyan`.
- `gray`.
- `dark_gray`.
- `light_red`.
- `light_green`.
- `light_yellow`.
- `light_blue`.
- `light_magenta`.
- `light_cyan`.
- `white`.
