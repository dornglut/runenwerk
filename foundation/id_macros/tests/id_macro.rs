use id_macros::id;
use static_assertions::assert_not_impl_any;

#[id]
pub struct MacroEntityId;

#[test]
fn generated_id_preserves_raw_value() {
    let id = MacroEntityId::try_from_raw(7).unwrap();

    assert_eq!(id.raw(), 7);
}

#[test]
fn generated_id_exposes_validating_constructor() {
    let id = MacroEntityId::try_from_raw(9).expect("non-zero id is valid");

    assert_eq!(id.raw(), 9);
    assert!(MacroEntityId::try_from_raw(0).is_err());
    assert!(MacroEntityId::try_from(0).is_err());
}

#[test]
fn generated_id_converts_to_and_from_underlying_typed_id() {
    let id = MacroEntityId::try_from_raw(11).unwrap();
    let typed: id::TypedId<__MacroEntityIdTag> = id.into();
    let restored = MacroEntityId::from(typed);

    assert_eq!(restored.raw(), 11);
}

#[test]
fn generated_id_has_no_infallible_raw_u64_conversion() {
    assert_not_impl_any!(MacroEntityId: From<u64>);
}
