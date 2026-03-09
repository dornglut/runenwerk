use super::PipelineKey;
use anyhow::{Result, anyhow, bail};
use std::collections::{BTreeMap, BTreeSet};

include!("render_graph_registry/internal/ids_and_registrations.rs");

include!("render_graph_registry/internal/spec.rs");

include!("render_graph_registry/internal/builders.rs");

include!("render_graph_registry/internal/registry.rs");

include!("render_graph_registry/internal/tests.rs");
