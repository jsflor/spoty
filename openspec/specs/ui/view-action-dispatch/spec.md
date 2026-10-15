# ui/view-action-dispatch Specification

## Purpose
Defines the declarative user interaction interface where immediate-mode views accept a unified view context and return strongly-typed action values to a centralized shell dispatcher.

## Requirements

### Requirement: Declarative View Return Value
Immediate-mode views SHALL express user interactions by returning `Option<ViewAction>` rather than invoking mutable callback closures passed through function arguments.

#### Scenario: User clicks track play button in view
- **GIVEN** an active view rendered with `ViewContext`
- **WHEN** user clicks the play button on a track row
- **THEN** the view function returns `Some(ViewAction::PlayTrack(track))` for that frame

#### Scenario: No user interaction during frame
- **GIVEN** an active view rendered with `ViewContext`
- **WHEN** no interactive widgets are triggered during the frame pass
- **THEN** the view function returns `None`

### Requirement: Centralized Shell Action Handling
The application shell SHALL handle all view actions through a single dispatch point, ensuring identical execution semantics across all routes without code duplication.

#### Scenario: Handling playlist creation action
- **GIVEN** any view returning `ViewAction::CreatePlaylistWithTrack(track_id)`
- **WHEN** the shell receives the action
- **THEN** it generates a new playlist, adds the track, and updates the catalog in one location regardless of which view triggered the action
