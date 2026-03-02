use crate::ast::*;
use std::path::Path;

pub fn parse_file(path: &Path) -> anyhow::Result<Message> {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("invalid file name: {}", path.display()))?
        .to_string();

    let content = std::fs::read_to_string(path)?;
    parse_str(&name, &content)
}

pub fn parse_str(name: &str, content: &str) -> anyhow::Result<Message> {
    let mut fields = Vec::new();
    let mut constants = Vec::new();

    for (i, raw_line) in content.lines().enumerate() {
        let line_no = i + 1;

        // Strip inline comment
        let line = match raw_line.find('#') {
            Some(pos) => &raw_line[..pos],
            None => raw_line,
        }
        .trim();

        if line.is_empty() {
            continue;
        }

        // Constants have '=' in them: TYPE NAME=value
        // Exclude '<=' which is part of bounded string syntax: string<=N field
        if let Some(eq) = line
            .find('=')
            .filter(|&i| i == 0 || line.as_bytes()[i - 1] != b'<')
        {
            let lhs = line[..eq].trim();
            let value = line[eq + 1..].trim().to_string();

            let (type_str, const_name) = split_type_name(lhs)
                .ok_or_else(|| anyhow::anyhow!("line {line_no}: malformed constant '{line}'"))?;

            let ty = match parse_primitive(type_str) {
                Some(t) => t,
                None => {
                    eprintln!(
                        "warning: line {line_no}: unsupported constant type '{type_str}', skipping"
                    );
                    continue;
                }
            };

            constants.push(Constant {
                name: const_name.to_string(),
                ty,
                value,
            });
            continue;
        }

        // Fields: TYPE[N] name  or  TYPE name  (optional default value after name is ignored)
        let (type_str, field_name) = split_type_name(line)
            .ok_or_else(|| anyhow::anyhow!("line {line_no}: malformed field '{line}'"))?;

        // field_name may have a trailing default value; take only the first token
        let field_name = field_name.split_whitespace().next().unwrap_or(field_name);

        let ty = parse_field_type(type_str).map_err(|e| {
            anyhow::anyhow!(
                "line {line_no}: field '{field_name}' has unsupported type '{type_str}': {e}"
            )
        })?;

        fields.push(Field {
            name: field_name.to_string(),
            ty,
        });
    }

    Ok(Message {
        name: name.to_string(),
        fields,
        constants,
    })
}

/// Split "TYPE name" into ("TYPE", "name"). Handles leading/trailing whitespace.
fn split_type_name(s: &str) -> Option<(&str, &str)> {
    let mut parts = s.splitn(2, char::is_whitespace);
    let type_str = parts.next()?.trim();
    let rest = parts.next()?.trim();
    if type_str.is_empty() || rest.is_empty() {
        return None;
    }
    Some((type_str, rest))
}

fn parse_field_type(s: &str) -> anyhow::Result<FieldType> {
    if let Some(bracket) = s.find('[') {
        let type_str = &s[..bracket];
        let rest = &s[bracket + 1..];
        let end = rest
            .find(']')
            .ok_or_else(|| anyhow::anyhow!("missing closing ']' in array type"))?;
        let n_str = &rest[..end];

        if n_str.is_empty() {
            return Err(anyhow::anyhow!(
                "variable-length arrays are not zero-copy compatible"
            ));
        }

        let n = n_str.parse::<usize>().map_err(|_| {
            anyhow::anyhow!("array bound '{n_str}' is not a valid positive integer")
        })?;
        let prim =
            parse_primitive(type_str).ok_or_else(|| anyhow::anyhow!("unknown primitive type"))?;
        Ok(FieldType::FixedArray(prim, n))
    } else {
        if let Some(bound) = s.strip_prefix("string<=") {
            // ROS2 bounded string: string<=N
            let n = bound.parse::<usize>().map_err(|_| {
                anyhow::anyhow!("bounded string upper bound '{bound}' is not a valid integer")
            })?;
            if n == 0 {
                return Err(anyhow::anyhow!("bounded string upper bound must be > 0"));
            }
            return Ok(FieldType::BoundedString(n));
        }
        if s == "string" {
            return Err(anyhow::anyhow!(
                "unbounded 'string' is not zero-copy compatible; use 'string<=N'"
            ));
        }
        Ok(FieldType::Primitive(parse_primitive(s).ok_or_else(
            || anyhow::anyhow!("unknown primitive type"),
        )?))
    }
}

fn parse_primitive(s: &str) -> Option<PrimitiveType> {
    match s {
        "bool" => Some(PrimitiveType::Bool),
        "byte" => Some(PrimitiveType::Byte),
        "char" => Some(PrimitiveType::Char),
        "int8" => Some(PrimitiveType::Int8),
        "uint8" => Some(PrimitiveType::Uint8),
        "int16" => Some(PrimitiveType::Int16),
        "uint16" => Some(PrimitiveType::Uint16),
        "int32" => Some(PrimitiveType::Int32),
        "uint32" => Some(PrimitiveType::Uint32),
        "int64" => Some(PrimitiveType::Int64),
        "uint64" => Some(PrimitiveType::Uint64),
        "float32" => Some(PrimitiveType::Float32),
        "float64" => Some(PrimitiveType::Float64),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{FieldType, PrimitiveType};

    #[test]
    fn test_basic_fields() {
        let msg = parse_str("Pose", "float64 x\nfloat64 y\nfloat64 z\n").unwrap();
        assert_eq!(msg.name, "Pose");
        assert_eq!(msg.fields.len(), 3);
        assert!(matches!(
            msg.fields[0].ty,
            FieldType::Primitive(PrimitiveType::Float64)
        ));
    }

    #[test]
    fn test_fixed_array() {
        let msg = parse_str("Data", "uint8[4] bytes\n").unwrap();
        assert!(matches!(
            msg.fields[0].ty,
            FieldType::FixedArray(PrimitiveType::Uint8, 4)
        ));
    }

    #[test]
    fn test_constant() {
        let msg = parse_str("Flags", "int32 MAX_SIZE=100\n").unwrap();
        assert_eq!(msg.constants.len(), 1);
        assert_eq!(msg.constants[0].name, "MAX_SIZE");
        assert_eq!(msg.constants[0].value, "100");
    }

    #[test]
    fn test_comment_stripping() {
        let msg = parse_str(
            "Point",
            "# full line comment\nfloat64 x  # inline comment\n",
        )
        .unwrap();
        assert_eq!(msg.fields.len(), 1);
        assert_eq!(msg.fields[0].name, "x");
    }

    #[test]
    fn test_bounded_string() {
        let msg = parse_str("Msg", "string<=64 label\n").unwrap();
        assert_eq!(msg.fields.len(), 1);
        assert!(matches!(msg.fields[0].ty, FieldType::BoundedString(64)));
    }

    #[test]
    fn test_unbounded_string_is_rejected() {
        let err = parse_str("Msg", "string name\nint32 id\n").unwrap_err();
        assert!(err
            .to_string()
            .contains("unbounded 'string' is not zero-copy compatible"));
    }

    #[test]
    fn test_dynamic_array_is_rejected() {
        let err = parse_str("Msg", "uint8[] data\nint32 id\n").unwrap_err();
        assert!(err
            .to_string()
            .contains("variable-length arrays are not zero-copy compatible"));
    }

    #[test]
    fn test_zero_bounded_string_is_rejected() {
        let err = parse_str("Msg", "string<=0 name\n").unwrap_err();
        assert!(err.to_string().contains("upper bound must be > 0"));
    }
}
