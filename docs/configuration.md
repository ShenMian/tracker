# Configuration

## Location

Default locations of configuration files on different platforms:

- **Linux/macOS**: `$HOME/.config/tracker/config.toml`.
- **Windows**: `%USERPROFILE%\.config\tracker\config.toml`.

## Default settings

```toml
[world_map]
follow_object = true
follow_smoothing = 0.3
show_terminator = true
show_visibility_area = true
time_delta_min = 1
lon_delta_deg = 10.0
map_color = "gray"
trajectory_color = "light_blue"
terminator_color = "dark_gray"
visibility_area_color = "yellow"

[satellite_groups]
cache_lifetime_min = 120
groups = [
    # Space Stations
    { label = "ISS", id = "1998-067A" },
    { label = "CSS", id = "2021-035A" },
    # Weather & Earth Resources Satellites
    { label = "Weather", group = "weather" },
    { label = "NOAA", group = "noaa" },
    { label = "GOES", group = "goes" },
    { label = "Earth resources", group = "resource" },
    { label = "Search & rescue", group = "sarsat" },
    { label = "Disaster monitoring", group = "dmc" },
    # Navigation Satellites
    { label = "GPS", group = "gps-ops" },
    { label = "GLONASS", group = "glo-ops" },
    { label = "Galileo", group = "galileo" },
    { label = "Beidou", group = "beidou" },
    # Scientific Satellites
    { label = "Space & Earth Science", group = "science" },
    { label = "Geodetic", group = "geodetic" },
    { label = "Engineering", group = "engineering" },
    { label = "Education", group = "education" },
    # Miscellaneous Satellites
    { label = "Military", group = "military" },
    { label = "Radar calibration", group = "radar" },
    { label = "CubeSats", group = "cubesat" },
]

[polar]
ground_station = { lat = <LAT>, lon = <LON>, alt = <ALT> }
```

## World Map Configuration

- `follow_object`: Whether to automatically center the map on the selected satellite.
- `follow_smoothing`: Smoothing factor for follow mode (0.0 = no movement, 1.0 = instant snap).
- `show_terminator`: Whether to display the day-night terminator line.
- `lon_delta_deg`: Longitude offset in degrees when scrolling the map horizontally.
- `time_delta_min`: Time step in minutes for time simulation controls.

## Satellite groups

Satellite TLE (Two-Line Element) data is retrieved from [CelesTrak](https://celestrak.org), a 501(c)(3) non-profit organization dedicated to providing free orbital data and resources to the space community.

- **Individual satellites**: Use the [Search Satellite Catalog](https://celestrak.org/satcat/search.php) to locate specific satellites. The `id` field should match the satellite's International Designator as listed in the catalog.
- **Function-based groups**: Complete list of available satellite categories can be found at [Current GP Element Sets](https://celestrak.org/NORAD/elements/). The `group` field in the configuration corresponds to these category identifiers.

## Color options

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
