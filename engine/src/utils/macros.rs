
#[macro_export]
macro_rules! shader {
    ($name:expr) => {
        include_str!(concat!(env!("../engine/resources/shaders/", $name)))
    };
}