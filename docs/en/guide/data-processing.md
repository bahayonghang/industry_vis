# Data Processing

Industry Vis includes built-in data processing features that can be configured and used in the group detail page.

## Processing Pipeline

Data processing is executed in the following order:

```
Raw Data → Outlier Removal → Resampling → Smoothing → Downsampling → Display
```

## Outlier Removal

Uses the **3σ rule** (three-sigma rule) to identify and remove outliers.

### Principle

For normally distributed data, approximately 99.7% of data points fall within μ±3σ. Data points outside this range are considered outliers.

```
Lower bound = μ - 3σ
Upper bound = μ + 3σ
```

### Use Cases

- Sensor misreadings
- Communication errors causing abnormal values
- Equipment failures producing spikes

### Configuration

Check **Outlier Removal** in the group detail page, and the system will automatically remove out-of-range data points.

## Time Series Resampling

Aggregates high-frequency data to specified time intervals using **mean aggregation**.

### Parameters

| Parameter | Description | Range |
|-----------|-------------|-------|
| Interval | Resampling time interval | 1-3600 seconds |

### Example

Original data (1-second sampling):
```
00:00:01 → 10.5
00:00:02 → 10.8
00:00:03 → 10.2
...
00:00:59 → 10.6
00:01:00 → 10.4
```

After resampling (60-second interval):
```
00:00:00 → 10.5 (mean of 00:00:00-00:00:59)
00:01:00 → 10.4 (mean of 00:01:00-00:01:59)
```

### Use Cases

- Viewing long-term trends
- Reducing data volume
- Eliminating high-frequency noise

## Smoothing Filter

Uses **moving average** algorithm to smooth data curves.

### Parameters

| Parameter | Description | Range |
|-----------|-------------|-------|
| Window | Sliding window size | 2-50 points |

### Algorithm

For each data point, calculate the mean of `window/2` points before and after:

```
smoothed[i] = mean(data[i - window/2 : i + window/2 + 1])
```

### Use Cases

- Eliminating random noise
- Making trends clearer
- Data preprocessing

## Auto Downsampling

When data volume exceeds the threshold, the system automatically performs uniform downsampling to prevent frontend rendering lag.

### Default Configuration

- Maximum **5000** data points per tag
- Uniform sampling when exceeded

### Algorithm

```
step = total_points / max_points
Keep points where index % step == 0
```

## Configuration Persistence

Data processing configuration is saved along with the group. When you open the group next time, the previous configuration is automatically loaded.

Configuration is saved in `tag_groups.toml`:

```toml
[[groups]]
id = "g1234567890"
name = "Temperature Monitoring"
tags = ["Tag1", "Tag2"]
created_at = "2024-01-01T00:00:00"
updated_at = "2024-01-01T00:00:00"

[groups.processing_config]
[groups.processing_config.outlier_removal]
enabled = true
method = "3sigma"

[groups.processing_config.resample]
enabled = true
interval = 60
method = "mean"

[groups.processing_config.smoothing]
enabled = false
method = "moving_avg"
window = 5
```

## Recommended Settings

| Scenario | Recommended Configuration |
|----------|--------------------------|
| Real-time Monitoring | No processing, keep raw data |
| Shift Analysis | Enable resampling (60s), optional smoothing |
| Trend Analysis | Enable resampling (300s) + smoothing (window 5) |
| Anomaly Diagnosis | Enable outlier removal, no smoothing |
