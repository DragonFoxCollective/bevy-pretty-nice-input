use bevy::prelude::*;

use crate::actions::ActionData;

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
#[relationship_target(relationship = BindingOf, linked_spawn)]
pub struct Bindings(#[relationship] Vec<Entity>);

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
#[relationship(relationship_target = Bindings)]
pub struct BindingOf(#[relationship] pub Entity);

#[derive(EntityEvent, Debug, Clone, Reflect)]
#[reflect(Debug, Clone)]
pub struct BindingUpdate {
    #[event_target]
    pub action: Entity,
    pub data: ActionData,
}
