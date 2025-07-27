# Tracker User Guide

Tracker is a terminal-based real-time satellite tracking and orbit prediction application. This guide will help you get started with using Tracker effectively.

For installation instructions, please refer to the [README.md](../README.md#installation) file.

## Getting Started

After installation, simply run `tracker` in your terminal to start the application:

```bash
tracker
```

## Interface Overview

The Tracker interface is divided into three main sections:

1. **World Map** (Left panel): Shows the current positions of satellites on a world map
2. **Object Information** (Top-right panel): Displays detailed information about the selected satellite
3. **Satellite Groups** (Bottom-right panel): Lists available satellite groups that can be enabled/disabled

## Basic Navigation

### Mouse Controls

- **Left-click** on a satellite in the world map to select it
- **Right-click** on the world map to deselect the current satellite
- **Left-click** on an entry in the satellite groups list to toggle it
- **Scroll** in the satellite groups or object information panels to navigate through the lists
- **Left-click** on a row in the object information panel to copy that information to your clipboard

### Keyboard Controls

- Press **Q** or **ESC** to quit the application
- Press **Ctrl+C** to quit the application

## Using Tracker

### 1. Selecting Satellite Groups

In the bottom-right panel, you'll see a list of satellite groups. By default, none are selected. To enable satellite tracking:

1. Left-click on the checkbox next to a satellite group (e.g., "CSS", "ISS", etc.)
2. The application will automatically download and update the orbital elements for that group
3. Satellites from the selected groups will appear on the world map

### 2. Selecting a Satellite

To view detailed information about a specific satellite:

1. Left-click on any satellite displayed on the world map
2. The satellite will be highlighted, and its detailed information will appear in the top-right panel
3. To deselect the satellite, right-click anywhere on the world map

### 3. Viewing Satellite Information

The top-right panel displays detailed information about the selected satellite:

- Name and identification numbers (NORAD ID, COSPAR ID)
- Current position (latitude, longitude, altitude)
- Speed and orbital period
- Location on Earth (city and country)
- Orbital elements (inclination, eccentricity, etc.)

### 4. Understanding the World Map

The world map shows:

- Current positions of all tracked satellites (marked with "+" symbols)
- The trajectory of the selected satellite (light blue line showing its predicted path)
- Different visual indicators:
  - Red "+" for unselected satellites
  - Blinking green "+" for the selected satellite
  - Reversed red "+" for hovered satellites

### 5. Copying Information

To copy information from the object information panel:

1. Left-click on any row in the information panel
2. The value in that row will be copied to your system clipboard
3. You can then paste it anywhere using your system's paste function

## Tips and Tricks

1. **Performance**: If you notice performance issues, try selecting fewer satellite groups
2. **Updates**: Satellite data automatically updates every 2 minutes
3. **Trajectory**: The trajectory line shows where the satellite will be in the future
4. **International Date Line**: Trajectory lines correctly handle crossing the international date line

## Troubleshooting

### No Satellites Displayed

- Ensure you've selected at least one satellite group in the bottom-right panel
- Check your internet connection (required for downloading satellite data)
- Wait a moment for satellite data to load after selecting a group

### Data Not Updating

- Tracker automatically updates satellite data every 2 minutes
- If data seems stale, try deselecting and reselecting the satellite group

### Application Crashes

- Ensure you're using the latest version of Tracker
- Check that your terminal supports the required features
- Report issues on the project's GitHub page with detailed error information

## Customization

Currently, Tracker has limited customization options. Future versions may include:

- Color scheme customization
- Update interval adjustment
- Additional satellite data sources

For feature requests or bug reports, please visit the [project's GitHub repository](https://github.com/ShenMian/tracker).
