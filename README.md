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
[[components.data_collection]]
id = "clickhouse"
file = "/var/edgee/components/clickhouse.wasm"
settings.endpoint = "https://UNIQUE_ID.clickhouse.cloud:8443"
settings.table = "edgee"
settings.password = "*******"
```


## Event Handling

First of all, create a new table with the following schema:

<details>
  <summary>Native JSON type (>=25.3)</summary>

  ```sql
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

</details>

<details>
  <summary>Experimental JSON type (<25.3)</summary>

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

</details>

<details>
  <summary>Experimental Object('json') type (<25.3)</summary>

  ```sql
  Set allow_experimental_object_type = 1;
  CREATE TABLE edgee (
      uuid UUID,
      event_type String,
      timestamp UInt64,
      timestamp_millis UInt64,
      timestamp_micros UInt64,
      consent Nullable(String),
      context Object('json'),
      data Object('json')
  ) ENGINE = MergeTree
  ORDER BY (timestamp_millis);
  ```

</details>

<details>
  <summary>If no JSON support at all</summary>

  ```sql
  CREATE TABLE edgee (
      uuid UUID,
      event_type String,
      timestamp UInt64,
      timestamp_millis UInt64,
      timestamp_micros UInt64,
      consent Nullable(String),
      context String,
      data String
  ) ENGINE = MergeTree
  ORDER BY (timestamp_millis);
  ```

</details>

### JSON fields

New records are ingested individually using `JSONEachRow`.

If your ClickHouse version supports `JSON` or `Object('json')` types, both `context` and `data` will contain additional JSON sub-fields, whose schema is automatically inferred at runtime.


Please note that:

- The sub-fields under `context` are always the same, so you can use queries such as `SELECT context.client.ip AS ip FROM edgee`
- The sub-fields under `data` depend on the value of `event_type`, so you can use queries such as:
  - `SELECT data.Track.name FROM edgee WHERE event_type = 'Track'`
  - `SELECT data.Page.path FROM edgee WHERE event_type = 'Page'`



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
[[components.data_collection]]
id = "clickhouse"
file = "/var/edgee/components/clickhouse.wasm"
settings.endpoint = "https://UNIQUE_ID.clickhouse.cloud:8443"
settings.table = "edgee"
settings.password = "*******"
```

Optional fields:

```toml
settings.database = "default" # optional
settings.username = "default" # optional
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
