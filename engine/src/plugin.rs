use crate::app::App;

pub trait Plugin: Send + Sync {
    fn build(&self, app: &mut App);
}
