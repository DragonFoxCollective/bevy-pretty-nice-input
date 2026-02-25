//! All the types that should be private but can't be because they're used in macros.

use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_event_chain::*;
pub use bevy_pretty_nice_input_derive::{Action, input, input_transition};

pub use crate::actions::{Action, ActionData, ActionOf, Actions, PrevAction2Data, PrevActionData};
pub use crate::binding_parts::{BindingPartData, BindingPartUpdate, BindingParts};
pub use crate::bindings::{BindingOf, BindingUpdate, Bindings};
pub use crate::conditions::{
    Condition, ConditionedBindingUpdate, Conditions, InputDisabled, InvalidateData, invalidate_pass,
};
pub use crate::events::{JustPressed, JustReleased, Pressed, Updated};

struct BindingPartUpdateOrData<'a> {
    binding_part_index: usize,
    update_value: f32,
    binding_parts: Box<dyn Fn(Entity) -> Result<f32> + 'a>,
    binding_parts_rel: &'a BindingParts,
}

impl BindingPartUpdateOrData<'_> {
    fn get(&self, index: usize) -> Result<f32> {
        if index == self.binding_part_index {
            Ok(self.update_value)
        } else {
            Ok((self.binding_parts)(
                self.binding_parts_rel.collection()[index],
            )?)
        }
    }
}

pub fn binding(
    update: On<BindingPartUpdate>,
    bindings: Query<(&BindingOf, &BindingParts)>,
    binding_parts: Query<&BindingPartData>,
    mut commands: Commands,
) -> Result {
    let (binding_of, binding_parts_rel) = bindings.get(update.binding)?;

    let binding_part_index = binding_parts_rel
        .collection()
        .iter()
        .position(|&e| e == update.binding_part)
        .ok_or(BevyError::from("Cannot find binding part in binding parts"))?;
    let update_or_data = BindingPartUpdateOrData {
        binding_part_index,
        update_value: update.value,
        binding_parts: Box::new(|entity| Ok(binding_parts.get(entity)?.0)),
        binding_parts_rel,
    };

    let data = if binding_parts_rel.collection().len() == 1 {
        ActionData::Axis1D(update_or_data.get(0)?)
    } else if binding_parts_rel.collection().len() == 2 {
        ActionData::Axis2D(Vec2::new(update_or_data.get(0)?, update_or_data.get(1)?))
    } else if binding_parts_rel.collection().len() == 3 {
        ActionData::Axis3D(Vec3::new(
            update_or_data.get(0)?,
            update_or_data.get(1)?,
            update_or_data.get(2)?,
        ))
    } else {
        return Err(BevyError::from(format!(
            "Binding has invalid number of parts: {}",
            binding_parts_rel.collection().len()
        )));
    };

    // debug!("Binding update received {:?}, {:?}", update.value, data);

    commands.trigger(BindingUpdate {
        action: binding_of.0,
        data,
    });

    Ok(())
}

pub fn action<A: Action>(
    binding_update: On<BindingUpdate>,
    mut actions: Query<(&ActionOf<A>, &mut PrevActionData)>,
    mut commands: Commands,
    relation: ChainEventRelation<ConditionedBindingUpdate>,
) -> Result {
    // debug!(
    //     "Action update received {} {:?}",
    //     ShortName::of::<A>(),
    //     binding_update.data
    // );

    let (action_of, mut prev) = actions.get_mut(binding_update.action)?;
    if prev.0 != binding_update.data {
        debug!(
            "Action {} changed from {:?} to {:?}",
            ShortName::of::<A>(),
            prev.0,
            binding_update.data
        );
    }
    prev.0 = binding_update.data;
    let input = action_of.0;

    ConditionedBindingUpdate {
        chain: relation.new_chain(binding_update.action),
        input,
        action: binding_update.action,
        data: binding_update.data,
    }
    .trigger(&mut commands);
    Ok(())
}

pub fn action_2<A: Action>(
    update: On<ConditionedBindingUpdate>,
    mut actions: Query<(&ActionOf<A>, &mut PrevAction2Data)>,
    mut commands: Commands,
) -> Result {
    let (action_of, mut prev) = actions.get_mut(update.action)?;
    let input = action_of.0;

    let data = update.data;
    let Some(prev_data) = prev.0.replace(data) else {
        debug!("Initialized {} with {:?}", ShortName::of::<A>(), data);
        return Ok(());
    };

    if data.as_1d().is_some() {
        debug!(
            "Action 2 update received {} {:?}",
            ShortName::of::<A>(),
            update.data
        );
    }

    if !data.is_zero() && prev_data.is_zero() {
        if data.as_1d().is_some() {
            debug!("Action just pressed {}", ShortName::of::<A>());
        }
        commands.trigger(JustPressed::<A> {
            input,
            data,
            _marker: PhantomData,
        });
    }
    if !data.is_zero() {
        commands.trigger(Pressed::<A> {
            input,
            data,
            _marker: PhantomData,
        });
    }
    commands.trigger(Updated::<A> {
        input,
        data,
        _marker: PhantomData,
    });
    if data.is_zero() && !prev_data.is_zero() {
        if data.as_1d().is_some() {
            debug!("Action just released {}", ShortName::of::<A>());
        }
        commands.trigger(JustReleased::<A> {
            input,
            _marker: PhantomData,
        });
    }

    Ok(())
}

pub fn action_2_invalidate<A: Action>(
    invalidate: On<InvalidateData>,
    mut actions: Query<&mut PrevAction2Data>,
) -> Result {
    let mut prev = actions.get_mut(invalidate.event_target())?;
    if prev.0.is_some() {
        debug!("Invalidating {}", ShortName::of::<A>());
        prev.0 = None;
    }
    Ok(())
}

pub fn action_initialize(
    actions: Query<(Entity, &PrevActionData, &PrevAction2Data)>,
    mut commands: Commands,
) -> Result {
    for (entity, prev_data, prev_data_2) in actions.iter() {
        if prev_data_2.0.is_none() {
            // debug!("Try initializing {} with {:?}", name, prev_data.0);
            commands.trigger(BindingUpdate {
                action: entity,
                data: prev_data.0,
            });
        }
    }
    Ok(())
}

pub fn action_enable<A: Action>(
    remove: On<Remove, InputDisabled>,
    inputs: Query<&Actions<A>>,
    actions: Query<&PrevActionData>,
    mut commands: Commands,
) -> Result {
    for &action in inputs.get(remove.entity)?.collection().iter() {
        let prev_data = actions.get(action)?;
        debug!(
            "Enabling input for {} using {:?}",
            ShortName::of::<A>(),
            prev_data.0
        );
        commands.trigger(BindingUpdate {
            action,
            data: prev_data.0,
        });
    }
    Ok(())
}

pub fn transition_on<A: Action, F: Bundle, T: Bundle + Default>(
    pressed: On<JustPressed<A>>,
    mut commands: Commands,
) {
    debug!(
        "Transitioning on {} => {}",
        ShortName::of::<F>(),
        ShortName::of::<T>()
    );
    commands
        .entity(pressed.input)
        .remove::<F>()
        .insert(T::default());
}

pub fn transition_off<A: Action, F: Bundle, T: Bundle + Default>(
    released: On<JustReleased<A>>,
    mut commands: Commands,
) {
    debug!(
        "Transitioning off {} => {}",
        ShortName::of::<F>(),
        ShortName::of::<T>()
    );
    commands
        .entity(released.input)
        .remove::<F>()
        .insert(T::default());
}

pub fn transition_target<A: Action, T: Component + TryFrom<ActionData, Error = BevyError>>(
    updated: On<Updated<A>>,
    mut commands: Commands,
) -> Result {
    commands
        .entity(updated.input)
        .insert(T::try_from(updated.data)?);
    Ok(())
}
