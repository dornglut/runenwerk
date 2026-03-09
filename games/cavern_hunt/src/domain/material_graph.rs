use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

include!("material_graph/internal/types_and_constants.rs");
include!("material_graph/internal/errors.rs");
include!("material_graph/internal/compile.rs");
include!("material_graph/internal/resolve_helpers.rs");
include!("material_graph/internal/compile_node.rs");
include!("material_graph/internal/tests.rs");
