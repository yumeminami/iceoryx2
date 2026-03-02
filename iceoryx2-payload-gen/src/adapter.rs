use crate::ir::CanonicalIr;
use crate::parser;
use std::path::Path;

pub fn adapt_file(path: &Path) -> anyhow::Result<CanonicalIr> {
    let source_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid file name: {}", path.display()))?
        .to_string();

    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("missing extension for {}", path.display()))?;

    let content = std::fs::read_to_string(path)?;
    adapt_str(&source_name, extension, &content)
}

pub fn adapt_str(source_name: &str, extension: &str, content: &str) -> anyhow::Result<CanonicalIr> {
    match extension {
        "msg" => {
            let msg = parser::parse_str(source_name, content)?;
            Ok(CanonicalIr {
                source_name: source_name.to_string(),
                messages: vec![msg],
            })
        }
        "srv" => {
            let sections = split_sections(content);
            if sections.len() != 2 {
                return Err(anyhow::anyhow!(
                    "service '{}' must contain exactly 2 sections separated by '---' (request/response), got {}",
                    source_name,
                    sections.len()
                ));
            }

            let request = parser::parse_str(&format!("{source_name}_Request"), &sections[0])?;
            let response = parser::parse_str(&format!("{source_name}_Response"), &sections[1])?;
            Ok(CanonicalIr {
                source_name: source_name.to_string(),
                messages: vec![request, response],
            })
        }
        "action" => {
            let sections = split_sections(content);
            if sections.len() != 3 {
                return Err(anyhow::anyhow!(
                    "action '{}' must contain exactly 3 sections separated by '---' (goal/result/feedback), got {}",
                    source_name,
                    sections.len()
                ));
            }

            let goal = parser::parse_str(&format!("{source_name}_Goal"), &sections[0])?;
            let result = parser::parse_str(&format!("{source_name}_Result"), &sections[1])?;
            let feedback = parser::parse_str(&format!("{source_name}_Feedback"), &sections[2])?;
            Ok(CanonicalIr {
                source_name: source_name.to_string(),
                messages: vec![goal, result, feedback],
            })
        }
        _ => Err(anyhow::anyhow!(
            "unsupported interface extension '.{}' for '{}'; expected .msg, .srv or .action",
            extension,
            source_name
        )),
    }
}

fn split_sections(content: &str) -> Vec<String> {
    let mut sections = vec![String::new()];

    for raw_line in content.lines() {
        if raw_line.trim() == "---" {
            sections.push(String::new());
            continue;
        }

        let current = sections
            .last_mut()
            .expect("sections always has at least one element");
        current.push_str(raw_line);
        current.push('\n');
    }

    sections
}

#[cfg(test)]
mod tests {
    use super::adapt_str;

    #[test]
    fn adapts_msg_into_single_message() {
        let ir = adapt_str("Pose", "msg", "float64 x\n").unwrap();
        assert_eq!(ir.source_name, "Pose");
        assert_eq!(ir.messages.len(), 1);
        assert_eq!(ir.messages[0].name, "Pose");
    }

    #[test]
    fn adapts_srv_into_request_and_response() {
        let ir = adapt_str("AddTwoInts", "srv", "int64 a\n---\nint64 sum\n").unwrap();
        assert_eq!(ir.messages.len(), 2);
        assert_eq!(ir.messages[0].name, "AddTwoInts_Request");
        assert_eq!(ir.messages[1].name, "AddTwoInts_Response");
    }

    #[test]
    fn adapts_action_into_goal_result_feedback() {
        let ir = adapt_str(
            "DoThing",
            "action",
            "int64 goal\n---\nbool ok\n---\nfloat32 progress\n",
        )
        .unwrap();
        assert_eq!(ir.messages.len(), 3);
        assert_eq!(ir.messages[0].name, "DoThing_Goal");
        assert_eq!(ir.messages[1].name, "DoThing_Result");
        assert_eq!(ir.messages[2].name, "DoThing_Feedback");
    }

    #[test]
    fn errors_when_srv_section_count_is_wrong() {
        let err = adapt_str("BadSrv", "srv", "int64 a\n").unwrap_err();
        assert!(err.to_string().contains("must contain exactly 2 sections"));
    }
}
