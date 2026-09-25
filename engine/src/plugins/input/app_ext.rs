use crate::app::App;
use runen_input::PhysicalKeyIdentity;

use super::{ActionState, InputState, input_integration_is_active};

pub trait AppActionBindingsExt {
    fn add_input_bindings<I>(&mut self, bindings: I) -> &mut Self
    where
        I: IntoIterator<Item = (&'static str, PhysicalKeyIdentity)>;
}

impl AppActionBindingsExt for App {
    fn add_input_bindings<I>(&mut self, bindings: I) -> &mut Self
    where
        I: IntoIterator<Item = (&'static str, PhysicalKeyIdentity)>,
    {
        if !input_integration_is_active(self.world()) {
            self.record_missing_capability("add_input_bindings", "InputFinalizePlugin");
            return self;
        }

        let mut actions = self
            .world_mut()
            .remove_resource::<ActionState>()
            .expect("InputFinalizePlugin activation should guarantee ActionState");
        {
            let input = self
                .world()
                .resource::<InputState>()
                .expect("InputFinalizePlugin activation should guarantee InputState");
            for (action, key) in bindings {
                actions.map_key(input, action.to_string(), key);
            }
        }
        self.world_mut().insert_resource(actions);
        self
    }
}
