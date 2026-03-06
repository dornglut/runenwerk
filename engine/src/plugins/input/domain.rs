use std::collections::{HashMap, HashSet};
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

include!("domain_internal/actions_and_bindings.rs");

include!("domain_internal/state.rs");

include!("domain_internal/tests.rs");
