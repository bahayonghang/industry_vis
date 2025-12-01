## ADDED Requirements

### Requirement: Data Table Display
The system SHALL display DataFrame data in a tabular format with efficient rendering.

#### Scenario: Basic table display
- **WHEN** query results are available
- **THEN** the system SHALL display data in a scrollable table
- **AND** show column headers with data types

#### Scenario: Virtual scrolling
- **WHEN** displaying large datasets (>1000 rows)
- **THEN** the system SHALL use virtual scrolling
- **AND** render only visible rows for performance

#### Scenario: Column resizing
- **WHEN** user drags column border
- **THEN** the column width SHALL adjust accordingly
- **AND** persist across sessions

#### Scenario: Column sorting
- **WHEN** user clicks column header
- **THEN** the table SHALL sort by that column
- **AND** toggle between ascending/descending

### Requirement: Time Range Selection
The system SHALL provide time range selection for querying historical data.

#### Scenario: Preset time ranges
- **WHEN** user opens time range selector
- **THEN** the system SHALL provide preset options:
  - 最近1小时
  - 最近6小时
  - 最近24小时
  - 最近7天
  - 自定义范围

#### Scenario: Custom time range
- **WHEN** user selects custom range
- **THEN** the system SHALL display date-time pickers for start and end time

#### Scenario: Time range query
- **WHEN** user confirms time range selection
- **THEN** the system SHALL query data within the specified DateTime range

### Requirement: Tag Selection
The system SHALL provide tag selection for filtering data by TagName.

#### Scenario: Tag list display
- **WHEN** user opens tag selector
- **THEN** the system SHALL display available TagName values from history table

#### Scenario: Multi-tag selection
- **WHEN** user selects multiple tags
- **THEN** the system SHALL filter data to include only selected tags

#### Scenario: Tag search
- **WHEN** user types in tag selector
- **THEN** the system SHALL filter tag list by search text

### Requirement: Line Chart Visualization
The system SHALL support line chart visualization for time-series data.

#### Scenario: Basic line chart
- **WHEN** user has selected time range and tags
- **THEN** the system SHALL render a line chart with DateTime as X-axis and TagVal as Y-axis

#### Scenario: Multiple tag series
- **WHEN** user selects multiple tags
- **THEN** the system SHALL render multiple lines (one per tag) with legend

#### Scenario: Zoom and pan
- **WHEN** user interacts with chart (scroll/drag)
- **THEN** the chart SHALL support zoom and pan operations

#### Scenario: Data point tooltip
- **WHEN** user hovers over a data point
- **THEN** the chart SHALL display tooltip with DateTime, TagName, TagVal, and TagQuality

### Requirement: Chart Configuration
The system SHALL provide basic configuration options for chart appearance.

#### Scenario: Legend toggle
- **WHEN** user clicks legend item
- **THEN** the corresponding series SHALL be hidden/shown

#### Scenario: Y-axis scale
- **WHEN** data has large value range
- **THEN** the system SHALL auto-scale Y-axis appropriately

### Requirement: Data Export
The system SHALL support exporting data to file formats.

#### Scenario: CSV export
- **WHEN** user requests CSV export
- **THEN** the system SHALL save DataFrame to CSV file
- **AND** open file save dialog for location selection

#### Scenario: Export with filters
- **WHEN** user exports filtered/processed data
- **THEN** the exported file SHALL contain the processed data
- **AND** NOT the original unfiltered data

### Requirement: Query History
The system SHALL maintain history of executed queries.

#### Scenario: History recording
- **WHEN** user executes a query
- **THEN** the query SHALL be added to history
- **AND** include timestamp and data source

#### Scenario: History recall
- **WHEN** user selects a history item
- **THEN** the query text SHALL be loaded into editor
- **AND** ready for re-execution

#### Scenario: History limit
- **WHEN** history exceeds configured limit
- **THEN** oldest entries SHALL be removed
