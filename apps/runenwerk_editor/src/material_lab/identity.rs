use asset::{AssetId, AssetSourceId, ImportJobId};
use editor_viewport::ExpressionProductId;
use material_graph::{MaterialGraphDocumentId, MaterialProductId};

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
const MATERIAL_VIEWPORT_PRODUCT_ID_BASE: u64 = 10_000;

pub fn material_document_id_for_source(
    asset_id: AssetId,
    source_id: AssetSourceId,
) -> MaterialGraphDocumentId {
    MaterialGraphDocumentId::new(nonzero_stable_hash(&[
        "material_document",
        &asset_id.raw().to_string(),
        &source_id.raw().to_string(),
    ]))
}

pub fn material_product_id_for_import_job(job_id: ImportJobId) -> MaterialProductId {
    MaterialProductId::new(job_id.raw())
}

pub fn material_preview_expression_product_id(
    product_id: MaterialProductId,
) -> ExpressionProductId {
    ExpressionProductId(
        product_id
            .raw()
            .saturating_add(MATERIAL_VIEWPORT_PRODUCT_ID_BASE),
    )
}

fn nonzero_stable_hash(parts: &[&str]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    if hash == 0 { 1 } else { hash }
}

#[cfg(test)]
mod tests {
    use super::*;
    use asset::{asset_id, asset_source_id, import_job_id};

    #[test]
    fn material_document_identity_is_stable_for_asset_source_pair() {
        assert_eq!(
            material_document_id_for_source(asset_id(1), asset_source_id(2)),
            material_document_id_for_source(asset_id(1), asset_source_id(2))
        );
        assert_ne!(
            material_document_id_for_source(asset_id(1), asset_source_id(2)),
            material_document_id_for_source(asset_id(1), asset_source_id(3))
        );
    }

    #[test]
    fn material_product_identity_uses_import_job_identity() {
        assert_eq!(
            material_product_id_for_import_job(import_job_id(42)).raw(),
            42
        );
        assert_eq!(
            material_preview_expression_product_id(MaterialProductId::new(42)),
            ExpressionProductId(10_042)
        );
    }
}
