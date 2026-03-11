const PRELUDE_RS: &str = include_str!("../src/prelude.rs");
const QUERY_MOD_RS: &str = include_str!("../src/query/mod.rs");
const WORLD_CORE_RS: &str = include_str!("../src/world/world_core_impl.rs");

#[test]
fn prelude_remains_gameplay_focused() {
    assert!(PRELUDE_RS.contains("Query"));
    assert!(PRELUDE_RS.contains("Res"));
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
fn world_query_public_signature_avoids_query_data_bound() {
    assert!(WORLD_CORE_RS.contains("QuerySpec"));
    assert!(!WORLD_CORE_RS.contains("pub fn query_state<Q: QueryData"));
}
