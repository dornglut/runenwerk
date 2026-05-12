//! App-owned asset pipeline runtime.
//!
//! Domain crates own asset and field-product contracts. The editor app owns
//! host paths, import execution, catalog publication timing, and provider
//! presentation.

pub mod catalog_runtime;
pub mod field_product_jobs;
pub mod import_jobs;
pub mod product_publication;

pub use catalog_runtime::*;
pub use field_product_jobs::*;
pub use import_jobs::*;
pub use product_publication::*;
