#[derive(Debug, Copy, Clone, PartialEq, Eq, Default, ecs::Component)]
pub struct Replicated;

pub trait Replicate:
    serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static
{
}

impl<T> Replicate for T where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static
{
}
