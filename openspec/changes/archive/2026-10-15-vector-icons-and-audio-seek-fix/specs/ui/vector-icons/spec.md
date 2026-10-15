# Specification: UI Vector Icons

## ADDED REQUIREMENTS

### Requirement: Vector Home Icon
The UI SHALL render the navigation Home icon using vector geometry (roof triangle and base structure) without relying on system or font unicode glyphs.

#### Scenario: Home button visible
- GIVEN the top navigation bar is rendered
- WHEN the Home pill button is drawn
- THEN it SHALL display a crisp vector house graphic that scales and aligns cleanly within the 36x36 circular button.

### Requirement: Vector Plus Icon
The UI SHALL render the "Create playlist" icon using perpendicular vector rectangles instead of unicode fullwidth characters.

#### Scenario: Create playlist button visible
- GIVEN the library sidebar header is rendered
- WHEN the Create Playlist button is drawn
- THEN it SHALL display a crisp vector plus symbol with equal arm lengths and rounded corners.

### Requirement: Vector Shuffle Icon
The UI SHALL render the shuffle action using vector crossed arrows with arrowheads rather than font arrow glyphs.

#### Scenario: Shuffle toggle button visible
- GIVEN the Spotify player bar is rendered
- WHEN the Shuffle button is drawn
- THEN it SHALL display two intersecting vector arrows with visible rightward arrowheads and an active green dot when shuffle is active.

### Requirement: Vector Close / Remove Icon
The UI SHALL render close and removal actions using antialiased vector cross strokes.

#### Scenario: Queue drawer header close button visible
- GIVEN the Queue drawer panel is open
- WHEN the close button is drawn
- THEN it SHALL display a vector 'X' composed of two diagonal line segments centered in the button area.

### Requirement: Vector Column Sort Indicators
The UI SHALL render column sorting direction indicators using vector triangle polygons.

#### Scenario: Column header active sort
- GIVEN the Songs view track list table
- WHEN a column is sorted ascending or descending
- THEN it SHALL display an upward or downward pointing vector triangle adjacent to the column label.
