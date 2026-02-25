use std::marker::PhantomData;

use crate::actions::{Action, ActionData};
use crate::bevy_event_chain::*;
use crate::bundles::{add_systems, observe};

use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

/// Filter layer that each binding's input will pass through so it may be discarded, invalidated, or changed.
pub trait Condition {
    fn bundle<A: Action>(&self) -> impl Bundle;
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
#[relationship_target(relationship = ConditionOf, linked_spawn)]
pub struct Conditions(#[relationship] Vec<Entity>);

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
#[relationship(relationship_target = Conditions)]
pub struct ConditionOf(#[relationship] Entity);

/// Event passed through each Condition to determine whether and how the [`Action`] is changed.
#[derive(RelatedChainEvent, Clone, Debug, Reflect)]
#[reflect(Clone, Debug)]
#[related_chain_event(relationship_target = Conditions, relationship = ConditionOf)]
pub struct ConditionedBindingUpdate {
    #[event_target]
    pub(crate) chain: RelatedEventChain,
    pub input: Entity,
    pub action: Entity,
    pub data: ActionData,
}

impl ConditionedBindingUpdate {
    pub fn trigger_next_with_data(&self, data: ActionData, commands: &mut Commands) {
        self.next().with_data(data).trigger(commands);
    }

    pub fn with_data(mut self, data: ActionData) -> Self {
        self.data = data;
        self
    }
}

#[allow(dead_code)]
#[deprecated(note = "TODO marker")]
fn condition_pass(update: On<ConditionedBindingUpdate>, mut commands: Commands) {
    update.trigger_next(&mut commands);
}

pub fn invalidate_pass(invalidate: On<InvalidateData>, mut commands: Commands) {
    invalidate.trigger_next(&mut commands);
}

/// [`Condition`] that only lets one valid input pass every duration.
#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct Cooldown {
    timer: Timer,
    prev: Option<ConditionedBindingUpdate>,
}

impl Cooldown {
    pub fn new(duration: f32) -> Self {
        let mut timer = Timer::from_seconds(duration, TimerMode::Once);
        timer.finish();
        Self { timer, prev: None }
    }
}

impl Condition for Cooldown {
    fn bundle<A: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 mut conditions: Query<(&Name, &mut Cooldown)>,
                 mut commands: Commands|
                 -> Result {
                    let (name, mut condition) = conditions.get_mut(update.event_target())?;

                    let data = update.data;
                    let prev_data = condition
                        .prev
                        .replace(update.clone())
                        .map(|prev| prev.data)
                        .unwrap_or(data);

                    if !data.is_zero() && prev_data.is_zero() {
                        if condition.timer.is_finished() {
                            debug!("{} Cooling down", name);
                            update.trigger_next(&mut commands);
                            update.trigger_next_with_data(data.zeroed(), &mut commands);
                        } else {
                            debug!("{} Re-cooling down", name);
                        }
                        condition.timer.set_mode(TimerMode::Repeating);
                    } else if data.is_zero() {
                        debug!("{} Un-cooling down", name);
                        condition.timer.set_mode(TimerMode::Once);
                        update.trigger_next(&mut commands);
                    }
                    Ok(())
                },
            ),
            observe(
                |invalidate: On<InvalidateData>, mut conditions: Query<&mut Cooldown>| -> Result {
                    let mut condition = conditions.get_mut(invalidate.event_target())?;
                    condition.prev = None;
                    Ok(())
                },
            ),
        )
    }
}

pub fn tick_cooldown(
    mut conditions: Query<(&Name, &mut Cooldown)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (name, mut condition) in conditions.iter_mut() {
        condition.timer.tick(time.delta());
        if condition.timer.is_finished()
            && condition.timer.mode() == TimerMode::Repeating
            && let Some(prev) = &condition.prev
        {
            debug!("{} Cooldown finished, sending {:?}", name, prev.data);
            prev.trigger_next(&mut commands);
            prev.trigger_next_with_data(prev.data.zeroed(), &mut commands);
        }
    }
}

/// [`Condition`] that only lets the input pass if the query filter matches.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct Filter<F: QueryFilter> {
    _marker: PhantomData<F>,
}

/// [`Condition`] that filters for [`ComponentBuffer<F>`].
pub type FilterBuffered<F> = Filter<With<ComponentBuffer<F>>>;

/// [`Condition`] that filters out [`InputDisabled`].
pub type IsInputEnabled = Filter<Without<InputDisabled>>;

impl<F: QueryFilter> Default for Filter<F> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<F: QueryFilter + Send + Sync + 'static> Condition for Filter<F> {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>, inputs: Query<(), F>, mut commands: Commands| {
                if inputs.get(update.input).is_ok() {
                    update.trigger_next(&mut commands);
                } else {
                    update.trigger_next_with_data(update.data.zeroed(), &mut commands);
                }
            },
        )
    }
}

/// [`Condition`] that only lets the input pass if the query filter matches. Otherwise, [invalidates](InvalidateData) the input.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InvalidatingFilter<F: QueryFilter> {
    _marker: PhantomData<F>,
}

/// [`Condition`] that [invalidates](InvalidateData) input with [`InputDisabled`].
pub type IsInputEnabledInvalidate = InvalidatingFilter<Without<InputDisabled>>;

impl<F: QueryFilter> Default for InvalidatingFilter<F> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<F: QueryFilter + Send + Sync + 'static> Condition for InvalidatingFilter<F> {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>, inputs: Query<(), F>, mut commands: Commands| {
                if inputs.get(update.input).is_ok() {
                    if ShortName::of::<F>().to_string() != "Without<InputDisabled>" {
                        debug!(
                            "Filter passed for {} filtering {}",
                            ShortName::of::<A>(),
                            ShortName::of::<F>()
                        );
                    }
                    update.trigger_next(&mut commands);
                } else {
                    InvalidateData::from(&*update).trigger_next(&mut commands);
                }
            },
        )
    }
}

/// [`Condition`] that acts as a rising edge filter.
#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct ButtonPress {
    pub threshold: f32,
    prev: Option<ActionData>,
}

impl ButtonPress {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            prev: None,
        }
    }
}

impl Default for ButtonPress {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            prev: None,
        }
    }
}

impl Condition for ButtonPress {
    fn bundle<A: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 mut commands: Commands,
                 mut conditions: Query<(&Name, &mut ButtonPress)>|
                 -> Result {
                    let (name, mut condition) = conditions.get_mut(update.event_target())?;

                    let data = update.data;
                    let prev_data = condition.prev.replace(update.data).unwrap_or(data);

                    if data.is_pressed_with(condition.threshold)
                        && !prev_data.is_pressed_with(condition.threshold)
                    {
                        debug!("{} Button Pressed", name);
                        update.trigger_next(&mut commands);
                        update.trigger_next_with_data(data.zeroed(), &mut commands);
                    } else if !data.is_pressed_with(condition.threshold) {
                        debug!("{} Button Passed", name);
                        update.trigger_next_with_data(data.zeroed(), &mut commands);
                    }
                    Ok(())
                },
            ),
            observe(
                |invalidate: On<InvalidateData>,
                 mut conditions: Query<&mut ButtonPress>|
                 -> Result {
                    let mut condition = conditions.get_mut(invalidate.event_target())?;
                    condition.prev = None;
                    Ok(())
                },
            ),
        )
    }
}

/// [`Condition`] that acts as a falling edge filter.
#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct ButtonRelease {
    pub threshold: f32,
    prev: Option<ActionData>,
}

impl ButtonRelease {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            prev: None,
        }
    }
}

impl Default for ButtonRelease {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            prev: None,
        }
    }
}

impl Condition for ButtonRelease {
    fn bundle<A: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 mut commands: Commands,
                 mut conditions: Query<&mut ButtonRelease>|
                 -> Result {
                    let mut condition = conditions.get_mut(update.event_target())?;

                    let data = update.data;
                    let prev_data = condition.prev.replace(update.data).unwrap_or(data);

                    if !data.is_pressed_with(condition.threshold)
                        && prev_data.is_pressed_with(condition.threshold)
                    {
                        update.trigger_next_with_data(prev_data, &mut commands);
                        update.trigger_next(&mut commands);
                    }
                    Ok(())
                },
            ),
            observe(
                |invalidate: On<InvalidateData>,
                 mut conditions: Query<&mut ButtonRelease>|
                 -> Result {
                    let mut condition = conditions.get_mut(invalidate.event_target())?;
                    condition.prev = None;
                    Ok(())
                },
            ),
        )
    }
}

/// [`Condition`] that inverts the update between zero and nonzero, using the last nonzero input when the current input is zero.
#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct Invert {
    prev_nonzero: Option<ActionData>,
}

impl Condition for Invert {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>,
             mut commands: Commands,
             mut conditions: Query<&mut Invert>|
             -> Result {
                let mut condition = conditions.get_mut(update.event_target())?;

                let data = update.data;
                let prev_good = condition.prev_nonzero;
                if !data.is_zero() {
                    condition.prev_nonzero = Some(data);
                }

                if data.is_zero() {
                    if let Some(prev) = prev_good {
                        update.trigger_next_with_data(prev, &mut commands);
                    } else {
                        // No idea what to do if there's no previous good input. Perhaps a Binding::inverted_default()?
                    }
                } else {
                    update.trigger_next_with_data(data.zeroed(), &mut commands);
                }
                Ok(())
            },
        )
    }
}

/// [`Condition`] that continues sending nonzero updates for a duration after the input stops being nonzero.
#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct InputBuffer {
    timer: Timer,
    prev: Option<ConditionedBindingUpdate>,
}

impl InputBuffer {
    pub fn new(duration: f32) -> Self {
        let mut timer = Timer::from_seconds(duration, TimerMode::Once);
        timer.finish();
        Self { timer, prev: None }
    }

    pub fn force_finish(&mut self) {
        let was_paused = self.timer.is_paused();
        self.timer.unpause();
        self.timer.finish();
        if was_paused {
            self.timer.pause();
        }
    }
}

impl Condition for InputBuffer {
    fn bundle<A: Action>(&self) -> impl Bundle {
        (
            observe(
                |update: On<ConditionedBindingUpdate>,
                 mut commands: Commands,
                 mut conditions: Query<&mut InputBuffer>|
                 -> Result {
                    let mut condition = conditions.get_mut(update.event_target())?;

                    let data = update.data;

                    update.trigger_next(&mut commands);
                    if !data.is_zero() {
                        condition.prev = Some(update.clone());
                        condition.timer.reset();
                        condition.timer.pause();
                    } else {
                        condition.timer.unpause();
                    }
                    Ok(())
                },
            ),
            observe(
                |invalidate: On<InvalidateData>,
                 mut conditions: Query<&mut InputBuffer>|
                 -> Result {
                    let mut condition = conditions.get_mut(invalidate.event_target())?;
                    condition.prev = None;
                    condition.force_finish();
                    Ok(())
                },
            ),
            observe(
                |reset: On<ResetBufferEvent>,
                 mut commands: Commands,
                 mut condition: Query<(&Name, &mut InputBuffer)>|
                 -> Result {
                    let (name, mut condition) = condition.get_mut(reset.event_target())?;
                    debug!("Resetting {} input buffer", name);
                    condition.force_finish();
                    if let Some(prev) = &condition.prev {
                        prev.trigger_next_with_data(prev.data.zeroed(), &mut commands);
                    }
                    Ok(())
                },
            ),
        )
    }
}

pub fn tick_input_buffer(
    mut conditions: Query<(&Name, &mut InputBuffer)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (name, mut condition) in conditions.iter_mut() {
        condition.timer.tick(time.delta());
        if !condition.timer.is_finished()
            && let Some(prev) = &condition.prev
        {
            debug!("{} Input Buffer active, sending {:?}", name, prev.data);
            prev.trigger_next(&mut commands);
        } else if condition.timer.just_finished()
            && let Some(prev) = &condition.prev
        {
            debug!(
                "{} Input Buffer finished, sending {:?}",
                name,
                prev.data.zeroed()
            );
            prev.trigger_next_with_data(prev.data.zeroed(), &mut commands);
        }
    }
}

#[derive(RelatedChainEvent, Clone, Debug, Reflect)]
#[reflect(Clone, Debug)]
#[related_chain_event(relationship_target = Conditions, relationship = ConditionOf)]
pub struct ResetBufferEvent {
    #[event_target]
    chain: RelatedEventChain,
}

impl From<&ConditionedBindingUpdate> for ResetBufferEvent {
    fn from(update: &ConditionedBindingUpdate) -> Self {
        Self {
            chain: update.chain.reversed(),
        }
    }
}

/// [`Condition`] that stops any previous [`InputBuffer`]s.
#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct ResetBuffer;

impl Condition for ResetBuffer {
    fn bundle<A: Action>(&self) -> impl Bundle {
        observe(
            |update: On<ConditionedBindingUpdate>, mut commands: Commands| {
                if !update.data.is_zero() {
                    ResetBufferEvent::from(&*update).trigger(&mut commands);
                }
                update.trigger_next(&mut commands);
            },
        )
    }
}

pub fn pass_reset_buffer(reset: On<ResetBufferEvent, ConditionOf>, mut commands: Commands) {
    reset.trigger_next(&mut commands);
}

/// Event sent from [`Condition`]s that forces the action to be invalidated.
///
/// Invalidated actions unset their previous state, so the next input will be used as the next previous state.
/// For example, if an action is invalidated and its binding is held down, it won't trigger [`JustPressed`](crate::prelude::JustPressed) regardless of whether it was held down before the invalidation.
#[derive(RelatedChainEvent, Clone, Debug, Reflect)]
#[reflect(Clone, Debug)]
#[related_chain_event(relationship_target = Conditions, relationship = ConditionOf)]
pub struct InvalidateData {
    #[event_target]
    chain: RelatedEventChain,
}

impl From<&ConditionedBindingUpdate> for InvalidateData {
    fn from(update: &ConditionedBindingUpdate) -> Self {
        Self {
            chain: update.chain.clone(),
        }
    }
}

/// Component that gets inserted when its component is inserted, and removed *after the timer expires* when its component is removed.
///
/// Component buffering is the component compliment to [input buffering](InputBuffer), and can be used for "coyote time" behavior.
///
/// Insert [`ComponentBuffer::observe`] to set this up.
///
/// This example adds `ComponentBuffer<Grounded>` when `Grounded` is added, and removes it 0.2 seconds after `Grounded` is removed:
///
/// ```rust
/// # use bevy::prelude::*;
/// # use bevy_pretty_nice_input::prelude::*;
/// #[derive(Component, Default)]
/// struct Grounded;
///
/// ComponentBuffer::<Grounded>::observe(0.2)
/// # ;
/// ```
///
/// For convenience, there is an alias for `Filter<With<ComponentBuffer<T>>>` that is [`FilterBuffered<T>`].
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct ComponentBuffer<T: Component> {
    timer: Timer,
    _marker: PhantomData<T>,
}

impl<T: Component> ComponentBuffer<T> {
    /// Bundle of observers to insert/remove [`ComponentBuffer`]
    pub fn observe(duration: f32) -> impl Bundle {
        (
            observe(move |add: On<Add, T>, mut commands: Commands| {
                let mut timer = Timer::from_seconds(duration, TimerMode::Once);
                timer.pause();
                commands.entity(add.entity).insert(ComponentBuffer::<T> {
                    timer,
                    _marker: PhantomData,
                });
            }),
            observe(
                |remove: On<Remove, T>, mut conditions: Query<&mut ComponentBuffer<T>>| -> Result {
                    let mut condition = conditions.get_mut(remove.entity)?;
                    condition.timer.reset();
                    condition.timer.unpause();
                    Ok(())
                },
            ),
            add_systems(PreUpdate, tick_component_buffer::<T>),
        )
    }
}

fn tick_component_buffer<T: Component>(
    mut buffers: Query<(Entity, &mut ComponentBuffer<T>)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut buffer) in buffers.iter_mut() {
        buffer.timer.tick(time.delta());
        if buffer.timer.is_finished() {
            commands.entity(entity).remove::<ComponentBuffer<T>>();
        }
    }
}

/// Marker component for input systems that should stop receiving updates.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct InputDisabled;
