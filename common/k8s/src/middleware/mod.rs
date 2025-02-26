use regex::Regex;

mod metadata;

pub use metadata::*;

lazy_static! {
    static ref K8S_REG: Regex = Regex::new(
        r#"^/var/log/containers/([a-z0-9A-Z\-.]+)_([a-z0-9A-Z\-.]+)_([a-z0-9A-Z\-.]+)-([a-z0-9]{64}).log$"#
    ).unwrap_or_else(|e| panic!("K8S_REG Regex::new() failed: {}", e));
}

struct ParseResult {
    pod_name: String,
    pod_namespace: String,
}

impl ParseResult {
    fn new(pod_name: String, pod_namespace: String) -> ParseResult {
        ParseResult {
            pod_name,
            pod_namespace,
        }
    }
}

fn parse_container_path(path: &str) -> Option<ParseResult> {
    let captures = K8S_REG.captures(path)?;
    Some(ParseResult::new(
        captures.get(1)?.as_str().into(),
        captures.get(2)?.as_str().into(),
    ))
}
