//! An action- and component-based input crate for Bevy.
//!
//! To get started, add the [`PrettyNiceInputPlugin`] plugin to your app,
//! define some [`Action`](crate::prelude::Action)s,
//! then add some [`input!`](crate::prelude::input) and [`input_transition!`](crate::prelude::input_transition) bundles to your player or input system entity.
//! Then you should be able to either observe action events like [`On<JustPressed<MyAction>>`](crate::prelude::JustPressed) or respond to changes in components.
//!
//! Check out the [examples](https://github.com/DragonFoxCollective/bevy_pretty_nice_input/tree/main/bevy_pretty_nice_input/examples).

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
#[doc(hidden)]
pub mod derive;
mod events;

/// Re-exports from [`bevy_event_chain`].
pub mod bevy_event_chain {
    pub use bevy_event_chain::{
        ChainEventRelation, EntityComponentTrigger, RelatedChainEvent, RelatedEventChain,
    };
}

/// All the common types someone should have to deal with.
///
/// Note that every other type used in the crate that has to be public because they're used in macros is in the hidden `derive` mod.
pub mod prelude {
    pub use crate::PrettyNiceInputPlugin;
    pub use crate::actions::{Action, ActionData};
    pub use crate::conditions::{
        ButtonPress, ButtonRelease, ComponentBuffer, Condition, ConditionedBindingUpdate, Cooldown,
        Filter, FilterBuffered, InputBuffer, InputDisabled, InvalidateData, InvalidatingFilter,
        Invert, IsInputEnabled, IsInputEnabledInvalidate, ResetBuffer,
    };
    pub use crate::events::{JustPressed, JustReleased, Pressed, Updated};
    pub use crate::{binding1d, binding2d};

    /// Derive for [`TryFrom<ActionData, Error = BevyError>`].
    ///
    /// Only works for structs with a single unnamed field.
    ///
    /// Requires the attribute `#[action_data(Axis_D)]`, where the parameter is a variant of [`ActionData`],
    /// and the unnamed field is of the corresponding type or something that the type can [`into`](core::convert::Into).
    pub use bevy_pretty_nice_input_derive::TryFromActionData;

    /// Derive for [`Action`].
    ///
    /// The optional `#[action]` attribute accepts `invalidate = true/false` to impl either `EnableFilter = IsInputEnabledInvalidate/IsInputEnabled`,
    /// defaulting to `true` and using `IsInputEnabledInvalidate`. See [invalidation](crate::prelude::InvalidateData).
    ///
    /// [`IsInputEnabledInvalidate`] works best for state-agnostic inputs, like opening/closing menus, where keeping the previous input would be harmful.
    /// [`IsInputEnabled`] works best for state machines, when controls can change while the input is disabled.
    pub use bevy_pretty_nice_input_derive::Action;

    /// Takes an [`Action`] type, some bindings, and optionally some conditions, and returns a component bundle for the input system entity.
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_pretty_nice_input::prelude::*;
    /// #[derive(Action)]
    /// struct Walk;
    ///
    /// input!(Walk, Axis2D[binding2d::wasd()], [Cooldown::new(0.5)])
    /// # ;
    /// ```
    pub use bevy_pretty_nice_input_derive::input;

    /// Uses [`input!`] and bundles of components to act as a state machine. Returns a component bundle.
    ///
    /// Note that components must impl [`Default`].
    ///
    /// In this example, the input system entity starts with the `Standing` component,
    /// and swaps it out for the `Walking` component when WASD is pressed,
    /// then swaps it back out for `Standing` again when WASD is released.
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_pretty_nice_input::prelude::*;
    /// #[derive(Component, Default)]
    /// struct Standing;
    /// #[derive(Component, Default)]
    /// struct Walking;
    ///
    /// // Spawn this in an entity
    /// input_transition!((Standing) <=> (Walking), Axis2D[binding2d::wasd()])
    /// # ;
    /// ```
    ///
    /// If a component is listed on both sides of a transition, it is not inserted or removed, but is merely used as a prerequisite for the transition to happen.
    /// For example, `Walking` will only be added/removed as long as `Standing` exists on the entity:
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_pretty_nice_input::prelude::*;
    /// # #[derive(Component, Default)]
    /// # struct Standing;
    /// # #[derive(Component, Default)]
    /// # struct Walking;
    /// input_transition!((Standing) <=> (Standing, Walking), Axis2D[binding2d::wasd()])
    /// # ;
    /// ```
    ///
    /// The transition arrow can either be unidirectional in either direction, or bidirectional for both directions.
    /// A left-to-right arrow means the transition will happen on [`JustPressed`],
    /// and a right-to-left arrow means the transition will happen on [`JustReleased`].
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_pretty_nice_input::prelude::*;
    /// # #[derive(Component, Default)]
    /// # struct Standing;
    /// # #[derive(Component, Default)]
    /// # struct Walking;
    /// input_transition!((Standing) <=> (Walking), Axis2D[binding2d::wasd()])
    /// # ;
    /// // is equivalent to
    /// (
    ///     input_transition!((Standing) => (Walking), Axis2D[binding2d::wasd()]),
    ///     input_transition!((Standing) <= (Walking), Axis2D[binding2d::wasd()]),
    /// )
    /// # ;
    /// ```
    ///
    /// Adding an [`Action`] before the bundle on a *to* side will use that [`Action`] for observers.
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_pretty_nice_input::prelude::*;
    /// # #[derive(Component, Default)]
    /// # struct Standing;
    /// #[derive(Action)]
    /// struct Jump;
    ///
    /// input_transition!((Standing) => Jump (Standing), Axis1D[binding1d::space()])
    /// # ;
    /// ```
    ///
    /// Components on the *from* side prefixed with `!` mean that the component must not be present to transition.
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_pretty_nice_input::prelude::*;
    /// # #[derive(Component, Default)]
    /// # struct Standing;
    /// # #[derive(Component, Default)]
    /// # struct Walking;
    /// # #[derive(Component, Default)]
    /// # struct Crouching;
    /// input_transition!((Standing, !Crouching) => (Walking), Axis2D[binding2d::wasd()])
    /// # ;
    /// ```
    ///
    /// A component on the *to* side may impl [`TryFrom<ActionData, Error = BevyError>`] (through the [`TryFromActionData`] derive) instead of [`Default`],
    /// and prefixed with `>` to insert the input state directly into the component.
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_pretty_nice_input::prelude::*;
    /// # #[derive(Component, Default)]
    /// # struct Standing;
    /// #[derive(Component, TryFromActionData)]
    /// #[action_data(Axis2D)]
    /// struct Walking(Vec2);
    ///
    /// input_transition!((Standing) => (>Walking), Axis2D[binding2d::wasd()])
    /// # ;
    /// ```
    ///
    /// Conditions may be used in the transition, same as in [`input!`], but only for unidirectional transitions.
    ///
    /// ```rust
    /// # use bevy::prelude::*;
    /// # use bevy_pretty_nice_input::prelude::*;
    /// # #[derive(Component, Default)]
    /// # struct Standing;
    /// # #[derive(Component, Default)]
    /// # struct Walking;
    /// # #[derive(Component, Default)]
    /// # struct Grounded;
    /// input_transition!((Standing) => (Walking), Axis2D[binding2d::wasd()], [Filter::<With<Grounded>>::default()])
    /// # ;
    /// ```
    pub use bevy_pretty_nice_input_derive::input_transition;
}

/// The plugin. Add this to your app or the crate won't work!
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
