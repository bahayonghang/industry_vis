# Data Export

## CSV Export

The system supports exporting query results to CSV format.

### Export Steps

1. Execute a data query
2. Click the "Export CSV" button
3. Select a save location in the file dialog
4. Enter filename and save

### CSV Format

Exported CSV files contain the following columns:

```csv
DateTime,TagName,TagVal,TagQuality
2024-01-01T00:00:00.000,Temperature_Sensor_1,25.5,Good
2024-01-01T00:00:01.000,Temperature_Sensor_1,25.6,Good
...
```

| Column | Description |
|--------|-------------|
| DateTime | ISO 8601 format timestamp |
| TagName | Tag name |
| TagVal | Value (4 decimal places) |
| TagQuality | Quality flag |

### Notes

- Exports current filtered data
- Large exports may take some time
- Files use UTF-8 encoding

## Future Plans

- [ ] Support selecting columns to export
- [ ] Support JSON export format
- [ ] Batch export functionality
