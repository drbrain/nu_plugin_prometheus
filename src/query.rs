use nu_protocol::{
    IntoInterruptiblePipelineData, PipelineData, Record, Signals, Span, Value, record,
};
use prometheus_http_query::response::{InstantVector, RangeVector, Sample};
use std::collections::HashMap;

fn add_labels(record: &mut Record, metric: &HashMap<String, String>, flatten: bool, span: Span) {
    if flatten {
        for (name, label) in metric {
            if name == "__name__" {
                continue;
            }

            record.push(name, Value::string(label, span));
        }
    } else {
        let mut labels = Record::new();
        for (name, label) in metric {
            if name == "__name__" {
                continue;
            }
            labels.push(name, Value::string(label, span));
        }

        record.insert("labels", Value::record(labels, span));
    }
}

pub fn matrix_to_value(
    matrix: Vec<RangeVector>,
    flatten: bool,
    span: Span,
    signals: &Signals,
) -> PipelineData {
    matrix
        .into_iter()
        .map(move |rv| {
            let metric = rv.metric();
            let values = rv
                .samples()
                .iter()
                .map(|value| scalar_to_value(value, span))
                .collect();

            let name = metric
                .get("__name__")
                .cloned()
                .unwrap_or("[UNKNOWN]".to_string());

            let mut record = record! {
                "name" => Value::string(name, span),
            };

            add_labels(&mut record, metric, flatten, span);

            record.insert("values", Value::list(values, span));

            Value::record(record, span)
        })
        .into_pipeline_data(span, signals.clone())
}

pub fn scalar_to_value(scalar: &Sample, span: Span) -> Value {
    Value::record(
        record! {
            "value" => Value::float(scalar.value(), span),
            "timestamp" => Value::float(scalar.timestamp(), span)
        },
        span,
    )
}

pub fn vector_to_value(
    vector: Vec<InstantVector>,
    flatten: bool,
    span: Span,
    signals: &Signals,
) -> PipelineData {
    vector
        .into_iter()
        .map(move |iv| {
            let metric = iv.metric();

            let name = metric
                .get("__name__")
                .cloned()
                .unwrap_or("[UNKNOWN]".to_string());

            let mut record = record! {
                "name" => Value::string(name, span),
            };

            add_labels(&mut record, metric, flatten, span);

            let value = Value::float(iv.sample().value(), span);
            record.insert("value", value);

            let timestamp = Value::float(iv.sample().timestamp(), span);
            record.insert("timestamp", timestamp);

            Value::record(record, span)
        })
        .into_pipeline_data(span, signals.clone())
}

#[cfg(test)]
mod test {
    use nu_protocol::{Signals, Span, Value, record};
    use prometheus_http_query::response::{InstantVector, RangeVector, Sample};
    use std::collections::HashMap;

    #[test]
    fn add_labels_flatten() {
        let mut metric = HashMap::new();
        metric.insert("job".into(), "prometheus".into());
        metric.insert("instance".into(), "localhost:9090".into());

        let mut record = record! {};

        super::add_labels(&mut record, &metric, true, Span::unknown());

        assert_eq!(
            Value::string("prometheus", Span::unknown()),
            record.get("job").unwrap().clone()
        );

        assert_eq!(
            Value::string("localhost:9090", Span::unknown()),
            record.get("instance").unwrap().clone()
        );
    }

    #[test]
    fn add_labels_no_flatten() {
        let mut metric = HashMap::new();
        metric.insert("job".into(), "prometheus".into());
        metric.insert("instance".into(), "localhost:9090".into());

        let mut record = record! {};

        super::add_labels(&mut record, &metric, false, Span::unknown());

        let expected = Value::record(
            record! {
                "job" => Value::string("prometheus", Span::unknown()),
                "instance" => Value::string("localhost:9090", Span::unknown()),
            },
            Span::unknown(),
        );

        assert_eq!(expected, record.get("labels").unwrap().clone());
    }

    #[test]
    fn matrix_to_value() {
        let data = r#"[
         {
            "metric" : {
               "__name__" : "up",
               "job" : "prometheus",
               "instance" : "localhost:9090"
            },
            "values" : [
               [ 1435781430.781, "1" ],
               [ 1435781445.781, "1" ],
               [ 1435781460.781, "1" ]
            ]
         },
         {
            "metric" : {
               "__name__" : "up",
               "job" : "node",
               "instance" : "localhost:9091"
            },
            "values" : [
               [ 1435781430.781, "0" ],
               [ 1435781445.781, "0" ],
               [ 1435781460.781, "1" ]
            ]
         }
      ]"#
        .as_bytes();
        let matrix: Vec<RangeVector> = serde_json::from_slice(data).unwrap();

        let result = super::matrix_to_value(matrix, false, Span::unknown(), &Signals::empty());

        let record = result
            .into_value(Span::unknown())
            .unwrap()
            .into_list()
            .unwrap()
            .first()
            .unwrap()
            .clone()
            .into_record()
            .unwrap();

        assert_eq!("up", record.get("name").unwrap().as_str().unwrap());

        let labels = record.get("labels").unwrap().as_record().unwrap();

        assert_eq!("prometheus", labels.get("job").unwrap().as_str().unwrap());

        let values = record.get("values").unwrap().as_list().unwrap();

        assert_eq!(3, values.len());
    }

    #[test]
    fn scalar_to_value() {
        let data = r#"[1716956024.754,"1"]"#.as_bytes();
        let scalar: Sample = serde_json::from_slice(data).unwrap();

        let result = super::scalar_to_value(&scalar, Span::unknown())
            .into_record()
            .unwrap();

        assert_eq!(1.0, result.get("value").unwrap().as_float().unwrap());
        assert_eq!(
            1716956024,
            result.get("timestamp").unwrap().as_float().unwrap() as u64
        );
    }

    #[test]
    fn vector_to_value() {
        let data = r#"[{"metric":{"__name__":"up","instance":"target.example","job":"job name"},"value":[1716956024.754,"1"]}]"#.as_bytes();
        let vector: Vec<InstantVector> = serde_json::from_slice(data).unwrap();

        let result = super::vector_to_value(vector, false, Span::unknown(), &Signals::empty())
            .into_value(Span::unknown())
            .unwrap()
            .into_list()
            .unwrap();

        let record = result.first().unwrap().as_record().unwrap();

        assert_eq!("up", record.get("name").unwrap().as_str().unwrap());

        let labels = record.get("labels").unwrap().as_record().unwrap();

        assert_eq!("job name", labels.get("job").unwrap().as_str().unwrap());

        let value = record.get("value").unwrap().as_float().unwrap();

        assert_eq!(1.0, value);

        let timestamp = record.get("timestamp").unwrap().as_float().unwrap();

        assert_eq!(1716956024, timestamp as u64);
    }
}
