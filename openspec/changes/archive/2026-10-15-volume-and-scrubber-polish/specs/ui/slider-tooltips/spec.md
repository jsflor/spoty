# ui/slider-tooltips Specification

## Purpose
Provides floating hover and drag tooltips for Spotify sliders to preview timestamp positions and volume percentages.

## Requirements

### Requirement: Timeline Hover Time Tooltip
The progress timeline slider SHALL render a floating tooltip pill above the cursor displaying the formatted time offset (`mm:ss`) corresponding to the hovered position.

#### Scenario: Hovering timeline at 2 minutes
- **GIVEN** an active track of 4 minutes
- **WHEN** user hovers halfway across the timeline bar
- **THEN** a tooltip bubble displaying "2:00" appears above the cursor

### Requirement: Volume Hover Percentage Tooltip
The volume slider SHALL render a floating tooltip pill above the cursor displaying the percentage value (`0%` to `100%`).

#### Scenario: Hovering volume at 75%
- **WHEN** user hovers over the volume slider at three-quarter width
- **THEN** a tooltip bubble displaying "75%" appears above the cursor
