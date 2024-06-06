use nu_protocol::{LabeledError, Span, Value};
use prometheus_http_query::Selector;
use regex::Regex;
use std::sync::OnceLock;

static LABEL_RE: OnceLock<Regex> = OnceLock::new();
static METRIC_RE: OnceLock<Regex> = OnceLock::new();

pub struct SelectorParser {}

impl SelectorParser {
    pub fn parse(input: &Value) -> Result<Selector, LabeledError> {
        let span = input.span();
        let parts: Vec<_> = input.as_str()?.splitn(2, '=').collect();

        if parts.is_empty() {
            return Err(LabeledError::new("Invalid selector").with_label("metric not found", span));
        }

        let selector = if parts.len() == 1 {
            let input = parts[0];
            let parts: Vec<_> = input.splitn(2, "!~").collect();

            if parts.len() == 1 {
                Selector::new().metric(check_metric(parts[0], span)?)
            } else {
                let label = check_label(parts[0], span)?;
                let value = parts[1];
                Selector::new().regex_ne(label, &value[1..value.len() - 1])
            }
        } else {
            let label = parts[0];
            let rest = parts[1];

            if label.ends_with('!') {
                let label = check_label(&label[0..label.len() - 1], span)?;

                Selector::new().ne(label, &rest[1..rest.len() - 1])
            } else if rest.starts_with('"') {
                Selector::new().eq(check_label(label, span)?, &rest[1..rest.len() - 1])
            } else if rest.starts_with('~') {
                Selector::new().regex_eq(check_label(label, span)?, &rest[2..rest.len() - 1])
            } else {
                return Err(LabeledError::new("Invalid selector")
                    .with_label("invalid metric matcher", span));
            }
        };

        Ok(selector)
    }
}

fn check_label(label: &str, span: Span) -> Result<&str, LabeledError> {
    let label_re = LABEL_RE.get_or_init(|| Regex::new(r#"\A[a-zA-Z_][a-zA-Z0-9_]*\z"#).unwrap());

    if label_re.is_match(label) {
        Ok(label)
    } else {
        Err(LabeledError::new("Invalid selector").with_label("invalid label name", span))
    }
}

fn check_metric(metric: &str, span: Span) -> Result<&str, LabeledError> {
    let metric_re =
        METRIC_RE.get_or_init(|| Regex::new(r#"\A[a-zA-Z_:][a-zA-Z0-9_:]*\z"#).unwrap());

    if metric_re.is_match(metric) {
        Ok(metric)
    } else {
        Err(LabeledError::new("Invalid selector").with_label("invalid metric name", span))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nu_protocol::Span;
    use rstest::rstest;

    #[rstest]
    #[case("a", true)]
    #[case("1", false)]
    #[case("a:", false)]
    fn check_label(#[case] label: &str, #[case] ok: bool) {
        assert_eq!(
            ok,
            super::check_label(label, Span::unknown()).is_ok(),
            "label: {label} ok: {ok}"
        );
    }

    #[rstest]
    #[case("a", true)]
    #[case("1", false)]
    #[case("a:", true)]
    #[case("a!", false)]
    fn check_metric(#[case] metric: &str, #[case] ok: bool) {
        assert_eq!(
            ok,
            super::check_metric(metric, Span::unknown()).is_ok(),
            "metric: {metric} ok: {ok}"
        );
    }

    #[test]
    fn eq() {
        let input = Value::string(r#"label="value""#, Span::unknown());
        let metric = SelectorParser::parse(&input).unwrap();

        assert_eq!(Selector::new().eq("label", "value"), metric);
    }

    #[test]
    fn metric() {
        let input = Value::string("metric", Span::unknown());
        let metric = SelectorParser::parse(&input).unwrap();

        assert_eq!(Selector::new().metric("metric"), metric);
    }

    #[test]
    fn ne() {
        let input = Value::string(r#"label!="value""#, Span::unknown());
        let metric = SelectorParser::parse(&input).unwrap();

        assert_eq!(Selector::new().ne("label", "value"), metric);
    }

    #[test]
    fn regex_eq() {
        let input = Value::string(r#"label=~"value""#, Span::unknown());
        let metric = SelectorParser::parse(&input).unwrap();

        assert_eq!(Selector::new().regex_eq("label", "value"), metric);
    }

    #[test]
    fn regex_ne() {
        let input = Value::string(r#"label!~"value""#, Span::unknown());
        let metric = SelectorParser::parse(&input).unwrap();

        assert_eq!(Selector::new().regex_ne("label", "value"), metric);
    }
}
