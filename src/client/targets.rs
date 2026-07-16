use crate::{Client, client::labeled_error, signals::run_with_signal};
use chrono::DateTime;
use nu_protocol::{
    IntoInterruptiblePipelineData, LabeledError, PipelineData, Signals, Span, Value, record,
};
use prometheus_http_query::{
    TargetState,
    response::{ActiveTarget, DroppedTarget},
};
use std::collections::HashMap;

pub struct Targets {
    client: prometheus_http_query::Client,
    target_state: Option<TargetState>,
}

impl Targets {
    pub fn new(client: prometheus_http_query::Client, target_state: Option<TargetState>) -> Self {
        Self {
            client,
            target_state,
        }
    }

    pub fn run(self, signals: &Signals, span: Span) -> Result<PipelineData, LabeledError> {
        let runtime = self.runtime()?;

        let Self {
            client,
            target_state,
        } = self;

        // NOTE: Doesn't impl Clone
        let target_state = match &target_state {
            Some(TargetState::Any) => Some(TargetState::Any),
            Some(TargetState::Dropped) => Some(TargetState::Dropped),
            Some(TargetState::Active) => Some(TargetState::Active),
            None => None,
        };

        // NOTE: Doesn't impl Clone
        let target_state2 = match &target_state {
            Some(TargetState::Any) => Some(TargetState::Any),
            Some(TargetState::Dropped) => Some(TargetState::Dropped),
            Some(TargetState::Active) => Some(TargetState::Active),
            None => None,
        };

        runtime.block_on(async {
            let targets = run_with_signal(signals, span, client.targets(target_state))
                .await?
                .map_err(|error| labeled_error(error, span))?;

            let value = match target_state2 {
                Some(TargetState::Active) => active(targets.active().to_vec(), span)
                    .into_pipeline_data(span, signals.clone()),
                Some(TargetState::Dropped) => dropped(targets.dropped().to_vec(), span)
                    .into_pipeline_data(span, signals.clone()),
                Some(TargetState::Any) | None => {
                    let active = active(targets.active().to_vec(), span)
                        .into_pipeline_data(span, signals.clone())
                        .into_value(span)?;
                    let dropped = dropped(targets.dropped().to_vec(), span)
                        .into_pipeline_data(span, signals.clone())
                        .into_value(span)?;

                    let record = record! {
                        "active" => active,
                        "dropped" => dropped,
                    };

                    PipelineData::value(Value::record(record, span), None)
                }
            };

            Ok(value)
        })
    }
}

impl Client for Targets {}

fn active(active: Vec<ActiveTarget>, span: Span) -> impl IntoInterruptiblePipelineData {
    active
        .into_iter()
        .map(move |target| {
            let record = record! {
                "discovered_labels" => hashmap_to_record(target.discovered_labels(), span),
                "global_url" => Value::string(target.global_url().as_str(), span),
                "health" => Value::string(target.health().to_string(), span),
                "labels" => hashmap_to_record(target.labels(), span),
                "last_error" => Value::string(target.last_error(), span),
                "last_scrape" => Value::date(
                    DateTime::from_timestamp(
                        target.last_scrape().unix_timestamp(),
                        target.last_scrape().nanosecond()).expect("invalid timestamp").fixed_offset(),
                    span
                ),
                "last_scrape_duration" => Value::duration((target.last_scrape_duration() * 1_000_000_000.0) as i64, span),
                "scrape_interval" => Value::duration(target.scrape_interval().whole_seconds() * 1_000_000_000, span),
                "scrape_pool" => Value::string(target.scrape_pool(), span),
                "scrape_timeout" => Value::duration(target.scrape_timeout().whole_seconds() * 1_000_000_000, span),
                "scrape_url" => Value::string(target.scrape_url().as_str(), span),
            };

            Value::record(record, span)
        })
}

fn dropped(dropped: Vec<DroppedTarget>, span: Span) -> impl IntoInterruptiblePipelineData {
    dropped.into_iter().map(move |target| {
        let record = record! {
            "discovered_labels" => hashmap_to_record(target.discovered_labels(), span),
        };

        Value::record(record, span)
    })
}

fn hashmap_to_record(labels: &HashMap<String, String>, span: Span) -> Value {
    let mut record = record! {};

    for (name, label) in labels {
        record.push(name, Value::string(label, span));
    }

    Value::record(record, span)
}

#[cfg(test)]
mod test {
    use nu_protocol::{IntoInterruptiblePipelineData, Signals, Span, Value};
    use prometheus_http_query::response::{ActiveTarget, DroppedTarget};

    #[test]
    fn active() {
        let data = r#"[
          {
            "discoveredLabels": {
              "__address__": "127.0.0.1:9090",
              "__metrics_path__": "/metrics",
              "__scheme__": "http",
              "job": "prometheus"
            },
            "labels": {
              "instance": "127.0.0.1:9090",
              "job": "prometheus"
            },
            "scrapePool": "prometheus",
            "scrapeUrl": "http://127.0.0.1:9090/metrics",
            "globalUrl": "http://example-prometheus:9090/metrics",
            "lastError": "",
            "lastScrape": "2017-01-17T15:07:44.723715405+01:00",
            "lastScrapeDuration": 0.050688943,
            "health": "up",
            "scrapeInterval": "1m",
            "scrapeTimeout": "10s"
          }
        ]"#
        .as_bytes();
        let active: Vec<ActiveTarget> = serde_json::from_slice(data).unwrap();

        let result = super::active(active, Span::unknown())
            .into_pipeline_data(Span::unknown(), Signals::empty())
            .into_value(Span::unknown())
            .unwrap();

        let record = result
            .into_list()
            .unwrap()
            .first()
            .unwrap()
            .clone()
            .into_record()
            .unwrap();

        let discovered_labels = record
            .get("discovered_labels")
            .unwrap()
            .as_record()
            .unwrap();

        assert_eq!(4, discovered_labels.len());
        assert_eq!(
            Value::string("127.0.0.1:9090", Span::unknown()),
            discovered_labels.get("__address__").unwrap().clone()
        );

        let labels = record.get("labels").unwrap().as_record().unwrap();

        assert_eq!(2, labels.len());
        assert_eq!(
            Value::string("127.0.0.1:9090", Span::unknown()),
            labels.get("instance").unwrap().clone()
        );

        assert_eq!(
            50688943,
            record
                .get("last_scrape_duration")
                .unwrap()
                .as_duration()
                .unwrap()
        );

        assert_eq!(
            "prometheus",
            record.get("scrape_pool").unwrap().as_str().unwrap()
        );

        assert_eq!(
            60_000_000_000,
            record
                .get("scrape_interval")
                .unwrap()
                .as_duration()
                .unwrap()
        );

        assert_eq!(
            10_000_000_000,
            record.get("scrape_timeout").unwrap().as_duration().unwrap()
        );
    }

    #[test]
    fn dropped() {
        let data = r#"[
          {
            "discoveredLabels": {
              "__address__": "127.0.0.1:9100",
              "__metrics_path__": "/metrics",
              "__scheme__": "http",
              "__scrape_interval__": "1m",
              "__scrape_timeout__": "10s",
              "job": "node"
            }
          }
        ]"#
        .as_bytes();

        let dropped: Vec<DroppedTarget> = serde_json::from_slice(data).unwrap();

        let result = super::dropped(dropped, Span::unknown())
            .into_pipeline_data(Span::unknown(), Signals::empty())
            .into_value(Span::unknown())
            .unwrap();

        let record = result
            .into_list()
            .unwrap()
            .first()
            .unwrap()
            .clone()
            .into_record()
            .unwrap();

        let discovered_labels = record
            .get("discovered_labels")
            .unwrap()
            .as_record()
            .unwrap();

        assert_eq!(6, discovered_labels.len());
        assert_eq!(
            Value::string("127.0.0.1:9100", Span::unknown()),
            discovered_labels.get("__address__").unwrap().clone()
        );
    }
}
