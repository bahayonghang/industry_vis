# Data Query

## Time Range Selection

The system provides two ways to select time ranges:

### Preset Time Ranges

- **Last 1 hour** - Query data from the past hour
- **Last 6 hours** - Query data from the past 6 hours
- **Last 24 hours** - Query data from the past 24 hours (default)
- **Last 7 days** - Query data from the past 7 days

### Custom Time Range

Select "Custom" to specify exact start and end times using the datetime picker.

## Tag Filtering

### Tag List

The system automatically fetches all available tag names from the database.

### Multi-Select

Select multiple tags simultaneously for querying and comparison.

### Search

Type keywords in the tag selector to quickly filter the tag list.

## Query Execution

1. Select a time range
2. Select tags to query (optional - queries all tags if none selected)
3. Data loads and displays automatically

## Data Table

Query results are displayed in a table:

| Column | Description |
|--------|-------------|
| Time | Data record timestamp |
| Tag Name | Data point tag name |
| Value | Measured value |
| Quality | Data quality flag |

### Table Features

- **Sorting** - Click column headers to sort
- **Filtering** - Tag column supports filtering
- **Pagination** - Default 50 records per page
- **Virtual Scrolling** - Efficiently handles large datasets

## Refresh Data

Click the "Refresh" button to reload data.
