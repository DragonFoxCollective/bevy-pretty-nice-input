//! 1-dimensional bindings, such as a single key press or joystick axis.

use bevy::ecs::spawn::SpawnableList;
use bevy::prelude::*;

use crate::binding_parts::{AxisDirection, BindingPartData, BindingPartOf, MouseScrollDirection};

/// Binding for a single key in the range \[0,1\].
pub fn key(key: KeyCode) -> impl SpawnableList<BindingPartOf> {
    Spawn((
        Name::new(format!("Key {:?}", key)),
        BindingPartData::default(),
        crate::binding_parts::Key(key),
    ))
}

/// Binding for two keys in the range \[-1,1\], with one being positive and the other negative.
pub fn key_axis(key_pos: KeyCode, key_neg: KeyCode) -> impl SpawnableList<BindingPartOf> {
    Spawn((
        Name::new(format!("Key Axis {:?} / {:?}", key_pos, key_neg)),
        BindingPartData::default(),
        crate::binding_parts::KeyAxis::new(key_pos, key_neg),
    ))
}

/// Binding for a single gamepad axis in the range \[-1,1\].
pub fn gamepad_axis(axis: GamepadAxis) -> impl SpawnableList<BindingPartOf> {
    Spawn((
        Name::new(format!("Gamepad Axis {:?}", axis)),
        BindingPartData::default(),
        crate::binding_parts::GamepadAxis(axis),
    ))
}

/// Binding for a single mouse button in the range \[0,1\].
pub fn mouse_button(button: MouseButton) -> impl SpawnableList<BindingPartOf> {
    Spawn((
        Name::new(format!("Mouse Button {:?}", button)),
        BindingPartData::default(),
        crate::binding_parts::MouseButton(button),
    ))
}

/// Binding for a single axis of mouse movement in the range \[-inf,inf\].
pub fn mouse_move_axis(axis: AxisDirection) -> impl SpawnableList<BindingPartOf> {
    Spawn((
        Name::new(format!("Mouse Move Axis {:?}", axis)),
        BindingPartData::default(),
        crate::binding_parts::MouseMoveAxis(axis),
    ))
}

/// Binding for a single direction of mouse scroll in the range \[0,inf\].
pub fn mouse_scroll(direction: MouseScrollDirection) -> impl SpawnableList<BindingPartOf> {
    Spawn((
        Name::new(format!("Mouse Scroll {:?}", direction)),
        BindingPartData::default(),
        crate::binding_parts::MouseScroll(direction),
    ))
}

/// Binding for a single axis of mouse scroll in the range \[-inf,inf\].
pub fn mouse_scroll_axis(axis: AxisDirection) -> impl SpawnableList<BindingPartOf> {
    Spawn((
        Name::new(format!("Mouse Scroll Axis {:?}", axis)),
        BindingPartData::default(),
        crate::binding_parts::MouseScrollAxis(axis),
    ))
}

pub fn space() -> impl SpawnableList<BindingPartOf> {
    key(KeyCode::Space)
}

pub fn left_shift() -> impl SpawnableList<BindingPartOf> {
    key(KeyCode::ShiftLeft)
}

pub fn left_ctrl() -> impl SpawnableList<BindingPartOf> {
    key(KeyCode::ControlLeft)
}

pub fn left_click() -> impl SpawnableList<BindingPartOf> {
    mouse_button(MouseButton::Left)
}

pub fn right_click() -> impl SpawnableList<BindingPartOf> {
    mouse_button(MouseButton::Right)
}

pub fn middle_click() -> impl SpawnableList<BindingPartOf> {
    mouse_button(MouseButton::Middle)
}

pub fn scroll_up() -> impl SpawnableList<BindingPartOf> {
    mouse_scroll(MouseScrollDirection::Up)
}

pub fn scroll_down() -> impl SpawnableList<BindingPartOf> {
    mouse_scroll(MouseScrollDirection::Down)
}

pub fn scroll_horizontal() -> impl SpawnableList<BindingPartOf> {
    mouse_scroll_axis(AxisDirection::X)
}

pub fn scroll_vertical() -> impl SpawnableList<BindingPartOf> {
    mouse_scroll_axis(AxisDirection::Y)
}
