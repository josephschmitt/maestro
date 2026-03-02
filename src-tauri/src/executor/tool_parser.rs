use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStartEvent {
    pub workspace_id: String,
    pub id: String,
    pub tool_name: String,
    pub input_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEndEvent {
    pub workspace_id: String,
    pub id: String,
    pub status: String,
    pub output_preview: Option<String>,
    pub output_full: Option<String>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}

#[derive(Debug)]
pub enum ParsedToolEvent {
    Start {
        id: String,
        tool_name: String,
        input_summary: String,
    },
    End {
        id: String,
        output_preview: Option<String>,
        output_full: Option<String>,
        error: Option<String>,
    },
}

pub fn parse_tool_line(line: &str) -> Option<ParsedToolEvent> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') || !trimmed.contains("\"maestro_tool\"") {
        return None;
    }

    let parsed: serde_json::Value = serde_json::from_str(trimmed).ok()?;

    match parsed.get("maestro_tool")?.as_str()? {
        "start" => {
            let id = parsed.get("id")?.as_str()?.to_string();
            let tool_name = parsed.get("tool_name")?.as_str()?.to_string();
            let input_summary = parsed
                .get("input_summary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Some(ParsedToolEvent::Start {
                id,
                tool_name,
                input_summary,
            })
        }
        "end" => {
            let id = parsed.get("id")?.as_str()?.to_string();
            let output_preview = parsed
                .get("output_preview")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let output_full = parsed
                .get("output_full")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let error = parsed
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Some(ParsedToolEvent::End {
                id,
                output_preview,
                output_full,
                error,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tool_start() {
        let line = r#"{"maestro_tool":"start","id":"t1","tool_name":"Read","input_summary":"src/main.rs"}"#;
        let event = parse_tool_line(line).unwrap();
        match event {
            ParsedToolEvent::Start {
                id,
                tool_name,
                input_summary,
            } => {
                assert_eq!(id, "t1");
                assert_eq!(tool_name, "Read");
                assert_eq!(input_summary, "src/main.rs");
            }
            _ => panic!("Expected Start event"),
        }
    }

    #[test]
    fn test_parse_tool_end() {
        let line = r#"{"maestro_tool":"end","id":"t1","output_preview":"fn main() {}"}"#;
        let event = parse_tool_line(line).unwrap();
        match event {
            ParsedToolEvent::End {
                id,
                output_preview,
                error,
                ..
            } => {
                assert_eq!(id, "t1");
                assert_eq!(output_preview, Some("fn main() {}".to_string()));
                assert!(error.is_none());
            }
            _ => panic!("Expected End event"),
        }
    }

    #[test]
    fn test_parse_tool_end_with_error() {
        let line = r#"{"maestro_tool":"end","id":"t2","error":"File not found"}"#;
        let event = parse_tool_line(line).unwrap();
        match event {
            ParsedToolEvent::End { id, error, .. } => {
                assert_eq!(id, "t2");
                assert_eq!(error, Some("File not found".to_string()));
            }
            _ => panic!("Expected End event"),
        }
    }

    #[test]
    fn test_parse_non_tool_line() {
        assert!(parse_tool_line("hello world").is_none());
        assert!(parse_tool_line(r#"{"other":"json"}"#).is_none());
        assert!(parse_tool_line("").is_none());
    }
}
