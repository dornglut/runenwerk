//! Canonical material literal to WGSL conversion.

use graph::NodeId;
use material_graph::{MaterialLiteral, MaterialValueType};

use super::super::MaterialShaderCompileError;

pub(super) fn output_variable(node_id: NodeId, output_name: &str) -> String {
    format!("n{}_{}", node_id.raw(), sanitize_identifier(output_name))
}

fn sanitize_identifier(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() {
            out.push(byte as char);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() || out.as_bytes()[0].is_ascii_digit() {
        out.insert(0, '_');
    }
    out
}

pub(super) fn wgsl_type(value_type: MaterialValueType) -> &'static str {
    match value_type {
        MaterialValueType::Float => "f32",
        MaterialValueType::Vec2 => "vec2<f32>",
        MaterialValueType::Vec3 => "vec3<f32>",
        MaterialValueType::Vec4 | MaterialValueType::Color => "vec4<f32>",
        MaterialValueType::Bool => "bool",
        MaterialValueType::ResourceTexture2D | MaterialValueType::ResourceTexture3D => "u32",
    }
}

pub(super) fn literal_to_wgsl(
    literal: &MaterialLiteral,
    value_type: MaterialValueType,
) -> Result<String, MaterialShaderCompileError> {
    match (literal, value_type) {
        (MaterialLiteral::Bool(value), MaterialValueType::Bool) => Ok(value.to_string()),
        (MaterialLiteral::Float(value), MaterialValueType::Float) => Ok(wgsl_float(value)?),
        (MaterialLiteral::Vec2(values), MaterialValueType::Vec2) => Ok(wgsl_vec("vec2", values)?),
        (MaterialLiteral::Vec3(values), MaterialValueType::Vec3) => Ok(wgsl_vec("vec3", values)?),
        (MaterialLiteral::Vec4(values), MaterialValueType::Vec4) => Ok(wgsl_vec("vec4", values)?),
        (MaterialLiteral::Color(values), MaterialValueType::Color) => Ok(wgsl_vec("vec4", values)?),
        _ => Err(MaterialShaderCompileError::InvalidLiteral {
            value: literal.canonical_component(),
            expected_type: value_type.label(),
        }),
    }
}

pub(super) fn canonical_value_to_wgsl(
    canonical: &str,
    value_type: MaterialValueType,
) -> Result<String, MaterialShaderCompileError> {
    match value_type {
        MaterialValueType::Float => canonical_float(canonical),
        MaterialValueType::Vec2 => canonical_vector_wgsl(canonical, "vec2", 2),
        MaterialValueType::Vec3 => canonical_vector_wgsl(canonical, "vec3", 3),
        MaterialValueType::Vec4 => canonical_vector_wgsl(canonical, "vec4", 4),
        MaterialValueType::Color => canonical_vector_wgsl(canonical, "vec4", 4),
        MaterialValueType::Bool => canonical_bool(canonical),
        MaterialValueType::ResourceTexture2D | MaterialValueType::ResourceTexture3D => {
            Err(MaterialShaderCompileError::InvalidLiteral {
                value: canonical.to_string(),
                expected_type: value_type.label(),
            })
        }
    }
}

fn canonical_float(canonical: &str) -> Result<String, MaterialShaderCompileError> {
    if let Some(value) = canonical.strip_prefix("integer:") {
        return wgsl_float(value);
    }
    if let Some(value) = length_prefixed_payload(canonical, "float")
        .or_else(|| length_prefixed_payload(canonical, "decimal"))
        .or_else(|| length_prefixed_payload(canonical, "text"))
    {
        return wgsl_float(value);
    }
    Err(MaterialShaderCompileError::InvalidLiteral {
        value: canonical.to_string(),
        expected_type: "float",
    })
}

fn canonical_bool(canonical: &str) -> Result<String, MaterialShaderCompileError> {
    if let Some(value) = canonical.strip_prefix("bool:") {
        return match value {
            "true" => Ok("true".to_string()),
            "false" => Ok("false".to_string()),
            _ => Err(MaterialShaderCompileError::InvalidLiteral {
                value: canonical.to_string(),
                expected_type: "bool",
            }),
        };
    }
    Err(MaterialShaderCompileError::InvalidLiteral {
        value: canonical.to_string(),
        expected_type: "bool",
    })
}

fn canonical_vector_wgsl(
    canonical: &str,
    wgsl_constructor: &str,
    count: usize,
) -> Result<String, MaterialShaderCompileError> {
    let values = if canonical.starts_with("text:") {
        let text = length_prefixed_payload(canonical, "text").ok_or_else(|| {
            MaterialShaderCompileError::InvalidLiteral {
                value: canonical.to_string(),
                expected_type: "vector",
            }
        })?;
        parse_text_vector(text, count)?
    } else {
        parse_canonical_vector(canonical, count)?
    };
    Ok(format!(
        "{wgsl_constructor}<f32>({})",
        values
            .iter()
            .map(|value| wgsl_float(value))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ")
    ))
}

fn parse_text_vector(value: &str, count: usize) -> Result<Vec<String>, MaterialShaderCompileError> {
    let values = value
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if values.len() == count {
        Ok(values)
    } else {
        Err(MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "vector",
        })
    }
}

fn parse_canonical_vector(
    canonical: &str,
    count: usize,
) -> Result<Vec<String>, MaterialShaderCompileError> {
    let mut cursor = 0usize;
    let label_end = canonical[cursor..].find(':').ok_or_else(|| {
        MaterialShaderCompileError::InvalidLiteral {
            value: canonical.to_string(),
            expected_type: "vector",
        }
    })? + cursor;
    let label = &canonical[cursor..label_end];
    if !matches!(label, "vec2" | "vec3" | "vec4" | "color") {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: canonical.to_string(),
            expected_type: "vector",
        });
    }
    cursor = label_end + 1;
    let encoded_count = read_decimal_until_colon(canonical, &mut cursor)?;
    if encoded_count != count {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: canonical.to_string(),
            expected_type: "vector",
        });
    }
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(read_length_prefixed_field(canonical, &mut cursor)?.to_string());
    }
    if cursor != canonical.len() {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: canonical.to_string(),
            expected_type: "vector",
        });
    }
    Ok(values)
}

fn length_prefixed_payload<'a>(canonical: &'a str, label: &str) -> Option<&'a str> {
    let prefix = format!("{label}:");
    let mut cursor = prefix.len();
    if !canonical.starts_with(&prefix) {
        return None;
    }
    let payload = read_length_prefixed_field(canonical, &mut cursor).ok()?;
    if cursor == canonical.len() {
        Some(payload)
    } else {
        None
    }
}

fn read_length_prefixed_field<'a>(
    value: &'a str,
    cursor: &mut usize,
) -> Result<&'a str, MaterialShaderCompileError> {
    let length = read_decimal_until_colon(value, cursor)?;
    let end =
        cursor
            .checked_add(length)
            .ok_or_else(|| MaterialShaderCompileError::InvalidLiteral {
                value: value.to_string(),
                expected_type: "length-prefixed field",
            })?;
    if end > value.len() || !value.is_char_boundary(end) {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "length-prefixed field",
        });
    }
    let payload = &value[*cursor..end];
    *cursor = end;
    if *cursor < value.len() {
        if value.as_bytes()[*cursor] != b':' {
            return Err(MaterialShaderCompileError::InvalidLiteral {
                value: value.to_string(),
                expected_type: "length-prefixed field",
            });
        }
        *cursor += 1;
    }
    Ok(payload)
}

fn read_decimal_until_colon(
    value: &str,
    cursor: &mut usize,
) -> Result<usize, MaterialShaderCompileError> {
    let start = *cursor;
    while *cursor < value.len() && value.as_bytes()[*cursor] != b':' {
        *cursor += 1;
    }
    if *cursor == value.len() || *cursor == start {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "length",
        });
    }
    let length = value[start..*cursor].parse::<usize>().map_err(|_| {
        MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "length",
        }
    })?;
    *cursor += 1;
    Ok(length)
}

fn wgsl_vec<const N: usize>(
    constructor: &str,
    values: &[String; N],
) -> Result<String, MaterialShaderCompileError> {
    Ok(format!(
        "{constructor}<f32>({})",
        values
            .iter()
            .map(|value| wgsl_float(value))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ")
    ))
}

fn wgsl_float(value: &str) -> Result<String, MaterialShaderCompileError> {
    let parsed = value
        .parse::<f32>()
        .map_err(|_| MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "float",
        })?;
    if !parsed.is_finite() {
        return Err(MaterialShaderCompileError::InvalidLiteral {
            value: value.to_string(),
            expected_type: "finite float",
        });
    }
    if value.contains('.') || value.contains('e') || value.contains('E') {
        Ok(value.to_string())
    } else {
        Ok(format!("{value}.0"))
    }
}
