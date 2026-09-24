use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn encode_ron_pretty<T: Serialize>(value: &T) -> Result<String, ron::Error> {
    let config = ron::ser::PrettyConfig::new()
        .separate_tuple_members(true)
        .enumerate_arrays(true);
    ron::ser::to_string_pretty(value, config)
}

pub fn decode_ron<T: DeserializeOwned>(source: &str) -> Result<T, ron::error::SpannedError> {
    ron::from_str(source)
}
