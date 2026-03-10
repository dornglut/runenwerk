// Owner: SDF Renderer Example - Input Binding Config
use crate::*;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub(crate) struct SdfInputBindingsConfig {
    pub(crate) bindings: Vec<SdfInputBindingConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct SdfInputBindingConfig {
    pub(crate) action: String,
    pub(crate) key: String,
}

impl Default for SdfInputBindingConfig {
    fn default() -> Self {
        Self {
            action: String::new(),
            key: String::new(),
        }
    }
}

impl Default for SdfInputBindingsConfig {
    fn default() -> Self {
        Self {
            bindings: vec![
                SdfInputBindingConfig {
                    action: ACTION_UP.to_string(),
                    key: "KeyR".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_DOWN.to_string(),
                    key: "KeyF".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_DEBUG_NEXT.to_string(),
                    key: "Tab".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_DEBUG_PREV.to_string(),
                    key: "Backquote".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_SPEED_UP.to_string(),
                    key: "KeyE".to_string(),
                },
                SdfInputBindingConfig {
                    action: ACTION_SPEED_DOWN.to_string(),
                    key: "KeyQ".to_string(),
                },
            ],
        }
    }
}
