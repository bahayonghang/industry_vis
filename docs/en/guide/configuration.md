# Configuration

Industry Vis uses TOML format configuration files.

## Configuration File Location

The `config.toml` file is located in the same directory as the application executable. During development, it's in the project root.

## Configuration Options

### Database Configuration `[database]`

```toml
[database]
# SQL Server address
server = "localhost"

# SQL Server port (default 1433)
port = 1433

# Database name
database = "控制器数据库"

# Username (SQL Server authentication)
username = "sa"

# Password
password = "your_password"
```

### Query Configuration `[query]`

```toml
[query]
# Default table name
default_table = "历史表"
```

## Default Values

If the configuration file doesn't exist, the system uses these defaults:

| Option | Default |
|--------|---------|
| server | localhost |
| port | 1433 |
| database | 控制器数据库 |
| username | sa |
| password | (empty) |
| default_table | 历史表 |

## In-App Configuration

You can also configure the database connection in the Settings page:

1. Click "Settings" in the top navigation
2. Fill in the database connection information
3. Click "Test Connection" to verify
4. Click "Save Configuration"

## Security Recommendations

- Don't commit `config.toml` with passwords to version control
- Use strong passwords in production
- Limit database user permissions to only necessary read access
