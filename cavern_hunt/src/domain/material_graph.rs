use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

include!("material_graph_internal/types_and_constants.rs");
include!("material_graph_internal/errors.rs");
include!("material_graph_internal/compile.rs");
include!("material_graph_internal/resolve_helpers.rs");
include!("material_graph_internal/compile_node.rs");
include!("material_graph_internal/tests.rs");
