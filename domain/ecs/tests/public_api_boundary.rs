const PRELUDE_RS: &str = include_str!("../src/prelude.rs");
const QUERY_MOD_RS: &str = include_str!("../src/query/mod.rs");
const BUNDLE_RS: &str = include_str!("../src/bundle.rs");
const COMPONENT_ACCESS_RS: &str = include_str!("../src/world/component/access.rs");
const COMPONENT_REGISTRATION_RS: &str = include_str!("../src/world/component/registration.rs");

#[test]
fn prelude_remains_gameplay_focused() {
    assert!(PRELUDE_RS.contains("Query"));
    assert!(PRELUDE_RS.contains("Res"));
    assert!(PRELUDE_RS.contains("ResView"));
    assert!(PRELUDE_RS.contains("ResMut"));
    assert!(PRELUDE_RS.contains("Commands"));
    assert!(PRELUDE_RS.contains("Runtime"));

    assert!(!PRELUDE_RS.contains("QueryAccess"));
    assert!(!PRELUDE_RS.contains("QueryTypeAccess"));
    assert!(!PRELUDE_RS.contains("QueryState"));
    assert!(!PRELUDE_RS.contains("SystemParam"));
    assert!(!PRELUDE_RS.contains("SystemParamError"));
}

#[test]
fn query_data_trait_stays_internal() {
    assert!(!QUERY_MOD_RS.contains("pub use traits_and_state::QueryData"));
    assert!(!QUERY_MOD_RS.contains("QueryData"));
}

#[test]
fn bundle_extension_boundary_is_unsafe_and_does_not_delegate_world_mutation() {
    assert!(BUNDLE_RS.contains("pub unsafe trait Bundle"));
    assert!(!BUNDLE_RS.contains("fn register(world: &mut World)"));
    assert!(!BUNDLE_RS.contains("fn insert(self, world: &mut World"));
    assert!(!BUNDLE_RS.contains("fn remove(world: &mut World"));
}

#[test]
fn obsolete_component_mutation_reach_through_is_not_public() {
    assert!(!COMPONENT_ACCESS_RS.contains("pub fn __insert_component"));
    assert!(!COMPONENT_ACCESS_RS.contains("pub fn __remove_component"));
    assert!(!COMPONENT_REGISTRATION_RS.contains("pub fn __register_component"));
}
