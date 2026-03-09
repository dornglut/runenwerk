use crate::plugins::shared::{
    ReloadStatusPayload, file_modified, should_poll, should_reload, watch_status_line,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

include!("shader_manager/internal/types.rs");

include!("shader_manager/internal/registry_impl.rs");

include!("shader_manager/internal/helpers.rs");

include!("shader_manager/internal/tests.rs");
