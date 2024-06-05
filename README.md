# nu_plugin_prometheus

A nushell plugin for querying prometheus

Supports:
* nushell 0.94.0
* Prometheus API
    * Instant queries
    * Range queryies
    * Target status
    * Label names
* Saved sources for mutual TLS authentication

## Usage

### Sources

A prometheus plugin can be queried directly with `--url`:

```nushell
"up" | prometheus query --source https://test.prometheus.example/
```

Nushell plugin configuration can be used to save configure prometheus sources
including mTLS.

```nushell
$env.config.plugins.prometheus = {
  sources: {
    prod: {
      url: "https://prod.prometheus.example/"
      cert: ( $env.HOME | path join ".config/nu_plugin_prometheus/user.crt" )
      key: ( $env.HOME | path join ".config/nu_plugin_prometheus/user.pk8.key" )
      cacert: ( $env.HOME | path join ".config/nu_plugin_prometheus/ca.crt" )
    }
  }
}
```

The key must be in PKCS#8 format. You can convert a PEM key with:

```nushell
openssl pkcs8 -topk8 -inform PEM -outform DER -in user.key -out user.pk8.key
```

Use `--source` or `-s` to use a configured source:

```nushell
"up" | prometheus query --source prod
```

### Queries

#### Instant

Pipe a prometheus query to `prometheus query` for an instant query:

```nushell
"up" | prometheus query --url https://prometheus.example:9090/
```

This will output a table:

|name|labels|value|timestamp|
|-|-|-|-|
|up|{job: prometheus, instance: prometheus.example:9090}|1|1435781451|
|up|{job: node, instance: prometheus.example:9100}|0|1435781451|

#### Range

A range query requires `--start`, `--end` and `--step` arguments:

```nushell
"up" | prometheus query range --url https://prometheus.example:9090/ --start ((date now) - 30sec) --end (date now) --step 15sec
```

|name|labels|values|
|-|-|-|
|up|{job: prometheus, instance: prometheus.example:9090}|[{value: 1, timestamp: 1435781430}, {value: 1, timestamp: 1435781445} {value: 1, timestamp: 1435781460}]|
|up|{job: node, instance: prometheus.example:9100}|[{value: 0, timestamp: 1435781430}, {value: 0, timestamp: 1435781445} {value: 1, timestamp: 1435781460}]|

#### Flattening labels

Adding `--flatten` will flatten labels into each row.

```nushell
"up" | prometheus query --url https://prometheus.example:9090/ --flatten
```

Outputs:

|name|instance|job|value|timestamp|
|-|-|-|-|-|
|up|prometheus.example:9090|prometheus|1|1435781451|
|up|prometheus.example:9100|job|0|1435781451|

If a metric uses "name" as a label it will overwrite the "name" column.

For a range query the values are not flattened.

### Label names

Retrieve labels names with:

```nushell
prometheus labels --url https://prometheus.example:9090/
```

Labels can be filtered by selector as input, or `--start`, `--end`.

### Targets

Retreive prometheus target discovery with:

```nushell
prometheus targets --url https://prometheus.example:9090/
```

This retrives targets in either the active or dropped states.  The `any`
argument alse retrieves both states.

Use `active`, or `dropped` to directly filter active or dropped targets.  This
will output only the selected state.

