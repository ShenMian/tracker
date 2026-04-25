# Configuration

## Location

Default configuration file locations on different platforms:

- **Linux/macOS**: `$HOME/.config/tracker/config.toml`.
- **Windows**: `%USERPROFILE%\.config\tracker\config.toml`.

## Default Settings

```toml
[world_map]
follow_object = true
follow_smoothing = 0.3
show_terminator = true
show_visibility_area = true
lon_delta_deg = 10.0
map_color = "gray"
trajectory_color = "light_blue"
terminator_color = "dark_gray"
visibility_area_color = "yellow"

[satellite_groups]
cache_lifetime_mins = 120
groups = [
    # Specific objects of interest
    { label = "ISS", id = "1998-067A" },
    { label = "CSS", id = "2021-035A" },
    
    # Special-interest satellites
    { label = "Last 30 Days", group = "last-30-days" },
    { label = "Space Stations", group = "stations" },
    { label = "100 Brightest", group = "visual" },
    { label = "Active", group = "active" },
    { label = "Analyst", group = "analyst" },
    
    # Weather & Earth resources satellites
    { label = "Weather", group = "weather" },
    { label = "Earth Resources", group = "resource" },
    { label = "SARSAT", group = "sarsat" },
    { label = "Disaster Monitoring", group = "dmc" },
    { label = "TDRSS", group = "tdrss" },
    { label = "ARGOS", group = "argos" },
    { label = "Planet", group = "planet" },
    { label = "Spire", group = "spire" },
    
    # Communications satellites
    { label = "GEO", group = "geo" },
    { label = "Intelsat", group = "intelsat" },
    { label = "SES", group = "ses" },
    { label = "Eutelsat", group = "eutelsat" },
    { label = "Telesat", group = "telesat" },
    { label = "Starlink", group = "starlink" },
    { label = "OneWeb", group = "oneweb" },
    { label = "Qianfan", group = "qianfan" },
    { label = "Hulianwang Digui", group = "hulianwang" },
    { label = "Kuiper", group = "kuiper" },
    { label = "Iridium NEXT", group = "iridium-NEXT" },
    { label = "Orbcomm", group = "orbcomm" },
    { label = "Globalstar", group = "globalstar" },
    { label = "Amateur Radio", group = "amateur" },
    { label = "SatNOGS", group = "satnogs" },
    { label = "Experimental Comm", group = "x-comm" },
    { label = "Other Comm", group = "other-comm" },
    
    # Navigation satellites
    { label = "GNSS", group = "gnss" },
    { label = "GPS Ops", group = "gps-ops" },
    { label = "GLONASS Ops", group = "glo-ops" },
    { label = "Galileo", group = "galileo" },
    { label = "Beidou", group = "beidou" },
    { label = "SBAS", group = "sbas" },
    
    # Scientific satellites
    { label = "Space & Earth Science", group = "science" },
    { label = "Geodetic", group = "geodetic" },
    { label = "Engineering", group = "engineering" },
    { label = "Education", group = "education" },
    
    # Miscellaneous satellites
    { label = "Military", group = "military" },
    { label = "Radar Calibration", group = "radar" },
    { label = "CubeSats", group = "cubesat" },
    
    # Debris
    { label = "Fengyun 1C Debris", group = "fengyun-1c-debris" },
    { label = "Iridium 33 Debris", group = "iridium-33-debris" },
    { label = "Cosmos 2251 Debris", group = "cosmos-2251-debris" },
]

[sky]
# ground_station = { name = <NAME>, position = {lat = <LAT_DEG>, lon = <LON_DEG>, alt = <ALT_KM>} }

[timeline]
time_delta_mins = 1
```

## World Map

- `follow_object`: Whether to automatically center the map on the selected satellite.
- `follow_smoothing`: Smoothing factor for follow mode (0.0 = no movement, 1.0 = instant snap).
- `show_terminator`: Whether to display the day-night terminator line.
- `lon_delta_deg`: Longitude offset in degrees when scrolling the map horizontally.
- `time_delta_mins`: Time step in minutes for time simulation controls.

## Satellite Groups

Satellite TLE (Two-Line Element) data is retrieved from [CelesTrak](https://celestrak.org), a 501(c)(3) non-profit organization dedicated to providing free orbital data and resources to the space community.

- **Individual satellites**: Use the [Search Satellite Catalog](https://celestrak.org/satcat/search.php) to locate specific satellites. The `id` field should match the satellite's International Designator as listed in the catalog.
- **Function-based groups**: Complete list of available satellite categories can be found at [Current GP Element Sets](https://celestrak.org/NORAD/elements/). The `group` field in the configuration corresponds to these category identifiers.

## Sky

The `ground_station.name` is optional. If not provided, the city name corresponding to the specified coordinates will be used.

## Color Options

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
