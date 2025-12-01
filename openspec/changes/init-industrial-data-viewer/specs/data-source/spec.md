## ADDED Requirements

### Requirement: Data Source Abstraction
The system SHALL provide an abstract interface for data sources that supports multiple database types.

#### Scenario: Adding new data source type
- **WHEN** implementing a new data source connector
- **THEN** the connector SHALL implement the DataSource trait
- **AND** expose query, test_connection, metadata, and list_tables methods

#### Scenario: Data source registration
- **WHEN** a data source connector is registered
- **THEN** the system SHALL make it available for connection configuration

### Requirement: SQL Server Connection
The system SHALL support connecting to Microsoft SQL Server databases using the tiberius library.

#### Scenario: Successful connection
- **WHEN** user provides valid SQL Server connection parameters
- **AND** invokes test connection
- **THEN** the system SHALL establish connection and return success

#### Scenario: Connection failure
- **WHEN** user provides invalid connection parameters
- **AND** invokes test connection
- **THEN** the system SHALL return descriptive error message
- **AND** SHALL NOT crash or hang

#### Scenario: SQL authentication
- **WHEN** user provides username and password
- **THEN** the system SHALL connect using SQL Server authentication

### Requirement: TOML Configuration File
The system SHALL support TOML configuration file for database connection settings.

#### Scenario: Loading configuration
- **WHEN** application starts
- **THEN** the system SHALL look for `config.toml` in application directory
- **AND** load database connection settings if file exists

#### Scenario: Default configuration
- **WHEN** `config.toml` does not exist or is incomplete
- **THEN** the system SHALL use default values:
  - server: localhost
  - port: 1433
  - database: 控制器数据库
  - default_table: 历史表

#### Scenario: Configuration structure
- **WHEN** user creates config.toml
- **THEN** the system SHALL accept the following structure:
  ```toml
  [database]
  server = "hostname"
  port = 1433
  database = "控制器数据库"
  username = "user"
  password = "pass"
  
  [query]
  default_table = "历史表"
  ```

#### Scenario: Configuration validation
- **WHEN** config.toml contains invalid values
- **THEN** the system SHALL display validation error
- **AND** indicate which fields are invalid

### Requirement: Query Execution
The system SHALL execute SQL queries against connected data sources and return results as Polars DataFrame.

#### Scenario: Default query
- **WHEN** application connects to database without explicit query
- **THEN** the system SHALL query the default table (历史表) from default database (控制器数据库)

#### Scenario: Successful query
- **WHEN** user executes a valid SQL query
- **THEN** the system SHALL return query results as DataFrame
- **AND** include column names and types

#### Scenario: Query error
- **WHEN** user executes an invalid SQL query
- **THEN** the system SHALL return the database error message
- **AND** SHALL NOT crash

#### Scenario: Query timeout
- **WHEN** query execution exceeds timeout threshold
- **THEN** the system SHALL cancel the query
- **AND** return timeout error message

### Requirement: History Table Schema Support
The system SHALL support the standard history table schema for industrial data.

#### Scenario: History table columns
- **WHEN** querying the history table
- **THEN** the system SHALL handle the following columns:
  - DateTime (datetime): timestamp
  - TagName (nchar 50): tag identifier
  - TagVal (real): numeric value
  - TagQuality (nchar 10): quality flag

#### Scenario: Tag filtering
- **WHEN** user selects specific tags
- **THEN** the system SHALL filter results by TagName column

### Requirement: Table Discovery
The system SHALL provide ability to list available tables and their schemas from connected data source.

#### Scenario: Listing tables
- **WHEN** user requests table list from data source
- **THEN** the system SHALL return list of tables with names and schemas

#### Scenario: Table schema inspection
- **WHEN** user requests schema for a specific table
- **THEN** the system SHALL return column names, types, and constraints
