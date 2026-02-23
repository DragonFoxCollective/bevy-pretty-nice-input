#![doc = include_str!("../README.md")]

use bevy::prelude::*;

mod actions;
pub mod binding1d;
pub mod binding2d;
mod binding_parts;
mod bindings;
pub mod bundles;
mod conditions;
#[cfg(feature = "debug_graph")]
pub mod debug_graph;
pub mod derive;
mod events;

pub mod bevy_event_chain {
    pub use bevy_event_chain::{
        ChainEventRelation, EntityComponentTrigger, RelatedChainEvent, RelatedEventChain,
    };
}

pub mod prelude {
    pub use crate::PrettyNiceInputPlugin;
    pub use crate::actions::Action;
    pub use crate::conditions::{
        ButtonPress, ButtonRelease, ComponentBuffer, Cooldown, Filter, FilterBuffered, InputBuffer,
        InvalidatingFilter, Invert, IsInputEnabled, IsInputEnabledInvalidate, ResetBuffer,
        invalidate_pass,
    };
    pub use crate::events::{JustPressed, JustReleased, Pressed, Updated};

    pub use bevy_pretty_nice_input_derive::{Action, TryFromActionData, input, input_transition};
}

#[derive(Default)]
pub struct PrettyNiceInputPlugin;

impl Plugin for PrettyNiceInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                (
                    binding_parts::binding_part_key,
                    binding_parts::binding_part_key_axis,
                    binding_parts::binding_part_gamepad_axis,
                    binding_parts::binding_part_mouse_button,
                    binding_parts::binding_part_mouse_move,
                    binding_parts::binding_part_mouse_scroll,
                    binding_parts::binding_part_mouse_scroll_axis,
                ),
                (
                    conditions::tick_cooldown,
                    conditions::tick_input_buffer,
                    derive::action_initialize,
                ),
            )
                .chain(),
        )
        .add_observer(conditions::pass_reset_buffer);

        #[cfg(feature = "debug_graph")]
        app.init_resource::<debug_graph::DebugGraph>();
    }
}
