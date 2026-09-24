use crate::app::App;

pub trait Plugin: Send + Sync {
    fn build(&self, app: &mut App);

    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
