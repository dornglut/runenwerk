use id_macros::id;

#[id]
#[cfg_attr(all(), derive(Clone))]
pub struct CfgAttrDeriveId;

fn main() {}
