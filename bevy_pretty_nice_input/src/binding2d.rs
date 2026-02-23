use bevy::ecs::spawn::SpawnableList;
use bevy::prelude::*;

use crate::binding_parts::{AxisDirection, BindingPartOf};
use crate::binding1d::*;

pub fn wasd() -> impl SpawnableList<BindingPartOf> {
    (
        key_axis(KeyCode::KeyD, KeyCode::KeyA),
        key_axis(KeyCode::KeyW, KeyCode::KeyS),
    )
}

pub fn arrow_keys() -> impl SpawnableList<BindingPartOf> {
    (
        key_axis(KeyCode::ArrowRight, KeyCode::ArrowLeft),
        key_axis(KeyCode::ArrowUp, KeyCode::ArrowDown),
    )
}

pub fn mouse_move() -> impl SpawnableList<BindingPartOf> {
    (
        mouse_move_axis(AxisDirection::X),
        mouse_move_axis(AxisDirection::Y),
    )
}
