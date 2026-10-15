# ui/queue-drawer Specification

## Purpose
Provides an interactive right-hand slide-out drawer panel to view Now Playing track information and manage upcoming queue items.

## Requirements

### Requirement: Queue Drawer Toggle
The player bar SHALL include a queue button (`≡`) on the right section that toggles the visibility of the Queue drawer panel.

#### Scenario: Opening and closing the queue drawer
- **WHEN** user clicks the queue icon in the player bar
- **THEN** the queue drawer opens and the button highlights in Spotify green (`#1ed760`); clicking again closes the drawer

### Requirement: Queue Content Display
The queue drawer SHALL display the currently playing track in a "Now Playing" card, followed by the upcoming tracks in a "Next Up" scrollable list.

#### Scenario: Inspecting queue
- **WHEN** the queue drawer is open
- **THEN** it displays the active track and lists all subsequent tracks in their expected playback order

### Requirement: Queue Track Removal and Clearing
The queue drawer SHALL provide controls to remove individual tracks from the upcoming queue and to clear all upcoming tracks in a single click.

#### Scenario: Clearing upcoming queue
- **WHEN** the user clicks "Clear Queue"
- **THEN** all upcoming queued tracks are removed while the currently playing track continues playing uninterrupted
