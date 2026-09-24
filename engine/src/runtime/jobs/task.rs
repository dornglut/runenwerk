use product::ProductJobDescriptor;

use crate::runtime::jobs::types::{RuntimeJobError, RuntimeJobGeneration, RuntimeJobKey};

pub type RuntimeJobResult<T> = Result<T, RuntimeJobError>;

pub trait RuntimeJob: Send + 'static {
    type Output: Send + 'static;

    fn product_job(&self) -> ProductJobDescriptor;

    fn generation(&self) -> RuntimeJobGeneration;

    fn key(&self) -> RuntimeJobKey {
        RuntimeJobKey::from_product_job(&self.product_job())
    }

    fn execute(self) -> RuntimeJobResult<Self::Output>;
}
