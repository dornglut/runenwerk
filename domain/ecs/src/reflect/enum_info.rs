//! File: domain/ecs/src/reflect/enum_info.rs
//! Purpose: Reflected no-payload enum metadata.

pub type EnumCurrentVariant = fn(&dyn std::any::Any) -> Option<&'static str>;
pub type EnumSetUnitVariant = fn(&mut dyn std::any::Any, &str) -> bool;

#[derive(Debug, Clone, Copy)]
pub struct EnumVariantInfo {
    pub symbol: &'static str,
    pub display_name: &'static str,
}

impl EnumVariantInfo {
    pub const fn new(symbol: &'static str, display_name: &'static str) -> Self {
        Self {
            symbol,
            display_name,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnumInfo {
    pub variants: &'static [EnumVariantInfo],
    pub current_variant: EnumCurrentVariant,
    pub set_unit_variant: EnumSetUnitVariant,
}

impl EnumInfo {
    pub const fn new(
        variants: &'static [EnumVariantInfo],
        current_variant: EnumCurrentVariant,
        set_unit_variant: EnumSetUnitVariant,
    ) -> Self {
        Self {
            variants,
            current_variant,
            set_unit_variant,
        }
    }

    pub fn variant_count(&self) -> usize {
        self.variants.len()
    }

    pub fn variant_named(&self, symbol: &str) -> Option<&'static EnumVariantInfo> {
        self.variants
            .iter()
            .find(|variant| variant.symbol == symbol)
    }
}
