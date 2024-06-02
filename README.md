# nu_plugin_prometheus

A nushell plugin for querying prometheus

Supports:
* nushell 0.94.0
* Prometheus instant queries
* Prometheus range queryies

## Usage

### Instant queries

Pipe a prometheus query to `prometheus query` for an instant query:

`"up" | prometheus query --url https://prometheus.example:9090/`

This will output a table:

|name|labels|value|timestamp|
|-|-|-|-|
|up|{job: prometheus, instance: prometheus.example:9090}|1|1435781451|
|up|{job: node, instance: prometheus.example:9100}|0|1435781451|

### Range queries

A range query requires `--start`, `--end` and `--step` arguments:

`"up" | prometheus query range --url https://prometheus.example:9090/ --start ((date now) - 30sec) --end (date now) --step 15sec`

|name|labels|values|
|-|-|-|
|up|{job: prometheus, instance: prometheus.example:9090}|[{value: 1, timestamp: 1435781430}, {value: 1, timestamp: 1435781445} {value: 1, timestamp: 1435781460}]|
|up|{job: node, instance: prometheus.example:9100}|[{value: 0, timestamp: 1435781430}, {value: 0, timestamp: 1435781445} {value: 1, timestamp: 1435781460}]|

### `--flatten`

Adding `--flatten` will flatten labels into each row.

`"up" | prometheus query --url https://prometheus.example:9090/ --flatten`

Outputs:

|name|instance|job|value|timestamp|
|-|-|-|-|-|
|up|prometheus.example:9090|prometheus|1|1435781451|
|up|prometheus.example:9100|job|0|1435781451|

For a range query the values are not flattened.

