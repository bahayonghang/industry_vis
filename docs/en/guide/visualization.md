# Data Visualization

## Line Chart

The system uses ECharts for time series data visualization.

### Multi-Series Display

- Each selected tag is displayed as a separate line
- Different tags use different colors
- Legend is shown at the bottom of the chart

### Interactive Features

#### Zoom

- **Inside Zoom** - Mouse wheel zooms in the chart area
- **Slider Zoom** - Drag the bottom slider to adjust display range

#### Pan

- Hold left mouse button and drag to pan the chart

#### Tooltips

Hovering over data points shows:
- Timestamp
- Values for each tag

### Y-Axis Auto-Scaling

The Y-axis automatically adjusts scale based on data range.

### Legend Interaction

- Click legend items to hide/show corresponding series
- Supports scrolling for many legend items

## View Switching

Switch between "Data Table" and "Trend Chart" tabs to change viewing mode.

## Best Practices

1. **Control Tag Count** - Displaying too many tags affects readability. Recommend no more than 10
2. **Choose Appropriate Time Ranges** - Very long ranges result in too many data points, affecting performance
3. **Use Zoom Features** - For long time ranges, use zoom to view details
