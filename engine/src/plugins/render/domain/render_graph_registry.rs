use super::PipelineKey;
use anyhow::{Result, anyhow, bail};
use std::collections::{BTreeMap, BTreeSet};

include!("render_graph_registry_internal/ids_and_registrations.rs");

include!("render_graph_registry_internal/spec.rs");

include!("render_graph_registry_internal/builders.rs");

include!("render_graph_registry_internal/registry.rs");

include!("render_graph_registry_internal/tests.rs");
