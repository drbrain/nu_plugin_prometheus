use nom::{
    branch::alt,
    bytes::complete::{tag, take_till, take_while, take_while1},
    combinator::{eof, map, recognize},
    error::context,
    multi::separated_list0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use nu_protocol::{LabeledError, Value};
use prometheus_http_query::Selector;

pub struct SelectorParser {}

impl SelectorParser {
    pub fn parse(input: &Value) -> Result<Selector, LabeledError> {
        let span = input.span();
        let input = input.as_str()?;

        let (_, selector) = selector(input)
            .map_err(|e| LabeledError::new("invalid selector").with_help(e.to_string()))?;

        Ok(selector)
    }
}

fn is_metric_label_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_metric_label_end(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_metric_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == ':'
}

fn is_metric_name_end(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == ':'
}

#[derive(Debug)]
enum Operation {
    Eq,
    Ne,
    RegexEq,
    RegexNe,
}

impl Operation {
    fn apply<'a>(self, selector: Selector<'a>, label: &'a str, value: &'a str) -> Selector<'a> {
        match self {
            Operation::Eq => selector.eq(label, value),
            Operation::Ne => selector.ne(label, value),
            Operation::RegexEq => selector.regex_eq(label, value),
            Operation::RegexNe => selector.regex_ne(label, value),
        }
    }
}

#[derive(Debug)]
struct LabelMatcher<'a> {
    label: &'a str,
    operation: Operation,
    value: &'a str,
}

impl<'a> LabelMatcher<'a> {
    fn apply(self, selector: Selector<'a>) -> Selector<'a> {
        self.operation.apply(selector, self.label, self.value)
    }
}

fn label(input: &str) -> IResult<&str, LabelMatcher, nom::error::VerboseError<&str>> {
    context(
        "label",
        map(
            tuple((metric_label, operation, label_value)),
            |(label, operation, value)| LabelMatcher {
                label,
                operation,
                value,
            },
        ),
    )(input)
}

fn labels(input: &str) -> IResult<&str, Vec<LabelMatcher>, nom::error::VerboseError<&str>> {
    context(
        "labels",
        delimited(tag("{"), separated_list0(tag(","), label), tag("}")),
    )(input)
}

fn label_value(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    delimited(tag("\""), take_till(|c| c == '"'), tag("\""))(input)
}

/// Matches a metric name `[a-zA-Z_][a-zA-Z0-9_]*`
fn metric_label(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    context(
        "metric label",
        recognize(preceded(
            take_while1(is_metric_label_start),
            take_while(is_metric_label_end),
        )),
    )(input)
}

/// Matches a metric name `[a-zA-Z_:][a-zA-Z0-9_:]*`
fn metric_name(input: &str) -> IResult<&str, &str, nom::error::VerboseError<&str>> {
    context(
        "metric name",
        recognize(preceded(
            take_while1(is_metric_name_start),
            take_while(is_metric_name_end),
        )),
    )(input)
}

fn operation(input: &str) -> IResult<&str, Operation, nom::error::VerboseError<&str>> {
    context(
        "operation",
        alt((
            map(tag("!="), |_| Operation::Ne),
            map(tag("=~"), |_| Operation::RegexEq),
            map(tag("!~"), |_| Operation::RegexNe),
            map(tag("="), |_| Operation::Eq),
        )),
    )(input)
}

fn selector(input: &str) -> IResult<&str, Selector, nom::error::VerboseError<&str>> {
    context(
        "selector",
        terminated(
            alt((
                map(labels, |labels| {
                    let mut selector = Selector::new();

                    for label_matcher in labels {
                        selector = label_matcher.apply(selector);
                    }

                    selector
                }),
                map(label, |label_matcher| label_matcher.apply(Selector::new())),
                map(tuple((metric_name, labels)), |(metric, labels)| {
                    let mut selector = Selector::new().metric(metric);

                    for label_matcher in labels {
                        selector = label_matcher.apply(selector);
                    }

                    selector
                }),
                map(metric_name, |name| Selector::new().metric(name)),
            )),
            eof,
        ),
    )(input)
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::error::VerboseErrorKind;
    use nu_protocol::Span;
    use rstest::rstest;

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
    fn metric_label_error() {
        let input = "0";

        let Err(nom::Err::Error(error)) = metric_label(input) else {
            unreachable!("input {input} must error");
        };

        let (error_input, VerboseErrorKind::Context(kind)) = error.errors.last().unwrap() else {
            unreachable!("error kind mismatch for {error:?}");
        };

        assert_eq!(&"0", error_input);
        assert_eq!(&"metric label", kind);
    }

    #[rstest]
    #[case("A0", "", "A0")]
    #[case("__name__", "", "__name__")]
    #[case("a0", "", "a0")]
    #[case("name_0_more", "", "name_0_more")]
    #[case("rule:name", ":name", "rule")]
    #[case("up", "", "up")]
    #[case("up{", "{", "up")]
    fn metric_label_ok(
        #[case] input: &str,
        #[case] expected_rest: &str,
        #[case] expected_parsed: &str,
    ) {
        let Ok((rest, parsed)) = metric_label(input) else {
            unreachable!("Unable to parse valid input {input}");
        };

        assert_eq!(
            expected_parsed, parsed,
            "parsed mismatch, expected {expected_parsed} got {parsed}"
        );
        assert_eq!(
            expected_rest, rest,
            "rest mismatch, expected {expected_rest} got {rest}"
        );
    }

    #[test]
    fn metric_name_error() {
        let input = "0";

        let Err(nom::Err::Error(error)) = metric_name(input) else {
            unreachable!("input {input} must error");
        };

        let (error_input, VerboseErrorKind::Context(kind)) = error.errors.last().unwrap() else {
            unreachable!("error kind mismatch for {error:?}");
        };

        assert_eq!(&"0", error_input);
        assert_eq!(&"metric name", kind);
    }

    #[rstest]
    #[case("A0", "", "A0")]
    #[case("__name__", "", "__name__")]
    #[case("a0", "", "a0")]
    #[case("name_0_more", "", "name_0_more")]
    #[case("rule:name", "", "rule:name")]
    #[case("up", "", "up")]
    #[case("up{", "{", "up")]
    fn metric_name_ok(
        #[case] input: &str,
        #[case] expected_rest: &str,
        #[case] expected_parsed: &str,
    ) {
        let Ok((rest, parsed)) = metric_name(input) else {
            unreachable!("Unable to parse valid input {input}");
        };

        assert_eq!(
            expected_parsed, parsed,
            "parsed mismatch, expected {expected_parsed} got {parsed}"
        );
        assert_eq!(
            expected_rest, rest,
            "rest mismatch, expected {expected_rest} got {rest}"
        );
    }

    #[test]
    fn ne() {
        let input = Value::string(r#"label!="value""#, Span::unknown());
        let metric = SelectorParser::parse(&input).unwrap();

        assert_eq!(Selector::new().ne("label", "value"), metric);
    }

    #[rstest]
    #[case("up", Selector::new().metric("up"))]
    #[case(r#"job="prometheus""#, Selector::new().eq("job", "prometheus"))]
    #[case(r#"job!="prometheus""#, Selector::new().ne("job", "prometheus"))]
    #[case(r#"job=~"p.+""#, Selector::new().regex_eq("job", "p.+"))]
    #[case(r#"job!~"p.+""#, Selector::new().regex_ne("job", "p.+"))]
    #[case(r#"up{job="prometheus"}"#, Selector::new().metric("up").eq("job", "prometheus"))]
    fn parse(#[case] input: &str, #[case] expected: Selector) {
        let value = Value::string(input, Span::unknown());

        let parsed = SelectorParser::parse(&value).unwrap();

        assert_eq!(expected, parsed, "input: {input} parsed: {parsed}");
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
