<div align="center">
<p align="center">
  <a href="https://www.edgee.cloud">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.cloud/img/component-dark.svg">
      <img src="https://cdn.edgee.cloud/img/component.svg" height="100" alt="Edgee">
    </picture>
  </a>
</p>
</div>

<h1 align="center">ClickHouse component for Edgee</h1>

[![Coverage Status](https://coveralls.io/repos/github/edgee-cloud/clickhouse-component/badge.svg)](https://coveralls.io/github/edgee-cloud/clickhouse-component)
[![GitHub issues](https://img.shields.io/github/issues/edgee-cloud/clickhouse-component.svg)](https://github.com/edgee-cloud/clickhouse-component/issues)
[![Edgee Component Registry](https://img.shields.io/badge/Edgee_Component_Registry-Public-green.svg)](https://www.edgee.cloud/edgee/clickhouse)


This component enables seamless integration between [Edgee](https://www.edgee.cloud) and [ClickHouse](https://clickhouse.com), allowing you to collect and forward analytics events to your ClickHouse tables.


## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `clickhouse.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[destinations.data_collection]]
id = "clickhouse"
file = "/var/edgee/components/clickhouse.wasm"
settings.endpoint = "https://UNIQUE_ID.clickhouse.cloud:8443"
settings.table = "edgee"
settings.username = "default"
settings.password = "*******"
```


## Event Handling

First of all, create a new table with this schema:

```sql
SET enable_json_type = 1;
CREATE TABLE edgee (
    uuid UUID,
    event_type String,
    timestamp UInt64,
    timestamp_millis UInt64,
    timestamp_micros UInt64,
    consent Nullable(String),
    context JSON,
    data JSON
) ENGINE = MergeTree
ORDER BY (timestamp_millis);
```

Please note that you can configure the table name in the component settings.

### Event Mapping
The component maps Edgee events to ClickHouse records as follows.

| Edgee Event | ClickHouse record | Description |
|-------------|----------------|-------------|
| Page        | A new record in the configured table | Full JSON dump of the Page event |
| Track       | A new record in the configured table | Full JSON dump of the Track event |
| User        | A new record in the configured table | Full JSON dump of the User event |


## Configuration Options

### Basic Configuration
```toml
[[destinations.data_collection]]
id = "clickhouse"
file = "/var/edgee/components/clickhouse.wasm"
settings.endpoint = "https://UNIQUE_ID.clickhouse.cloud:8443"
settings.table = "edgee"
settings.username = "default"
settings.password = "*******"
```


### Event Controls
Control which events are forwarded to ClickHouse:
```toml
settings.edgee_page_event_enabled = true   # Enable/disable page view tracking
settings.edgee_track_event_enabled = true  # Enable/disable custom event tracking
settings.edgee_user_event_enabled = true   # Enable/disable user identification
```


## Development

### Building from Source
Prerequisites:
- [Rust](https://www.rust-lang.org/tools/install)

Build command:
```bash
edgee component build
```

Test command:
```bash
make test
```

Test coverage command:
```bash
make test.coverage[.html]
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)
