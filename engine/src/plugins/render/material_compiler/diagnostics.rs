//! Material compiler diagnostics.

use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaterialShaderCompileError {
    UnsupportedIrVersion {
        found: u32,
        expected: u32,
    },
    MissingOutputNode,
    DuplicateOutputNode,
    MissingInput {
        node_id: u64,
        input: String,
    },
    MissingNodeValue {
        node_id: u64,
        key: String,
    },
    MissingResourceBinding {
        node_id: u64,
        key: String,
    },
    MissingConnectedOutput {
        node_id: u64,
        input: String,
        source_node_id: u64,
        output: String,
    },
    InvalidNodeContract {
        node_id: u64,
        message: String,
    },
    InvalidLiteral {
        value: String,
        expected_type: &'static str,
    },
    InvalidSceneMaterialTable(String),
    InvalidWgsl(String),
}

impl fmt::Display for MaterialShaderCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedIrVersion { found, expected } => write!(
                formatter,
                "unsupported material IR contract version {found}; expected {expected}"
            ),
            Self::MissingOutputNode => formatter.write_str("material IR has no pbr.output node"),
            Self::DuplicateOutputNode => {
                formatter.write_str("material IR has multiple pbr.output nodes")
            }
            Self::MissingInput { node_id, input } => {
                write!(
                    formatter,
                    "material node {node_id} is missing input '{input}'"
                )
            }
            Self::MissingNodeValue { node_id, key } => {
                write!(
                    formatter,
                    "material node {node_id} is missing value '{key}'"
                )
            }
            Self::MissingResourceBinding { node_id, key } => write!(
                formatter,
                "material node {node_id} is missing resolved resource binding '{key}'"
            ),
            Self::MissingConnectedOutput {
                node_id,
                input,
                source_node_id,
                output,
            } => write!(
                formatter,
                "material node {node_id} input '{input}' references missing output {source_node_id}.{output}"
            ),
            Self::InvalidNodeContract { node_id, message } => {
                write!(
                    formatter,
                    "material node {node_id} has invalid compiler contract: {message}"
                )
            }
            Self::InvalidLiteral {
                value,
                expected_type,
            } => write!(
                formatter,
                "material literal '{value}' cannot be compiled as {expected_type}"
            ),
            Self::InvalidSceneMaterialTable(message) => {
                write!(formatter, "invalid scene material table: {message}")
            }
            Self::InvalidWgsl(message) => write!(formatter, "generated WGSL is invalid: {message}"),
        }
    }
}

impl Error for MaterialShaderCompileError {}
