use bevy::input::gamepad::GamepadAxisChangedEvent;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::prelude::*;

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
#[relationship_target(relationship = BindingPartOf, linked_spawn)]
pub struct BindingParts(#[relationship] Vec<Entity>);

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
#[relationship(relationship_target = BindingParts)]
pub struct BindingPartOf(#[relationship] Entity);

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default, Debug)]
pub struct BindingPartData(pub f32);

#[derive(EntityEvent, Debug, Reflect)]
#[reflect(Debug)]
pub struct BindingPartUpdate {
    #[event_target]
    pub binding: Entity,
    pub binding_part: Entity,
    pub value: f32,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct Key(pub KeyCode);

pub fn binding_part_key(
    mut binding_parts: Query<(Entity, &Key, &BindingPartOf, &mut BindingPartData)>,
    bindings: Query<&Name>,
    mut commands: Commands,
    mut key: MessageReader<KeyboardInput>,
) -> Result {
    for message in key.read() {
        for (entity, key, binding_part_of, mut data) in binding_parts.iter_mut() {
            let name = bindings.get(binding_part_of.0)?;
            let value = message.state.is_pressed() as u8 as f32;
            if key.0 == message.key_code && !message.repeat && data.0 != value {
                debug!(
                    "{} Key {:?} value changed from {} to {}",
                    name, key.0, data.0, value
                );
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
    Ok(())
}

#[derive(Debug, Reflect)]
pub struct KeyAxisPart {
    pub key: KeyCode,
    pub is_pressed: bool,
}

impl KeyAxisPart {
    pub fn new(key: KeyCode) -> KeyAxisPart {
        KeyAxisPart {
            key,
            is_pressed: false,
        }
    }

    pub fn is_pressed_f32(&self) -> f32 {
        self.is_pressed as u8 as f32
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct KeyAxis {
    pub pos: KeyAxisPart,
    pub neg: KeyAxisPart,
}

impl KeyAxis {
    pub fn new(pos: KeyCode, neg: KeyCode) -> KeyAxis {
        KeyAxis {
            pos: KeyAxisPart::new(pos),
            neg: KeyAxisPart::new(neg),
        }
    }
}

pub fn binding_part_key_axis(
    mut binding_parts: Query<(Entity, &mut KeyAxis, &BindingPartOf, &mut BindingPartData)>,
    mut commands: Commands,
    mut key_axis: MessageReader<KeyboardInput>,
) {
    for message in key_axis.read() {
        for (entity, mut key_axis, binding_part_of, mut data) in binding_parts.iter_mut() {
            if message.repeat {
                continue;
            }

            if key_axis.pos.key == message.key_code {
                key_axis.pos.is_pressed = message.state.is_pressed();
            } else if key_axis.neg.key == message.key_code {
                key_axis.neg.is_pressed = message.state.is_pressed();
            } else {
                continue;
            };

            let value = key_axis.pos.is_pressed_f32() - key_axis.neg.is_pressed_f32();
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct GamepadAxis(pub bevy::prelude::GamepadAxis);

pub fn binding_part_gamepad_axis(
    mut binding_parts: Query<(Entity, &GamepadAxis, &BindingPartOf, &mut BindingPartData)>,
    mut commands: Commands,
    mut gamepad_axis: MessageReader<GamepadAxisChangedEvent>,
) {
    for message in gamepad_axis.read() {
        for (entity, gamepad_axis, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = message.value;
            if gamepad_axis.0 == message.axis && data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct MouseButton(pub bevy::prelude::MouseButton);

pub fn binding_part_mouse_button(
    mut binding_parts: Query<(Entity, &MouseButton, &BindingPartOf, &mut BindingPartData)>,
    mut commands: Commands,
    mut mouse_button: MessageReader<MouseButtonInput>,
) {
    for message in mouse_button.read() {
        for (entity, mouse_button, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = message.state.is_pressed() as u8 as f32;
            if mouse_button.0 == message.button && data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

#[derive(Debug, Reflect)]
#[reflect(Debug)]
pub enum AxisDirection {
    X,
    Y,
}

impl AxisDirection {
    pub fn index(&self) -> usize {
        match self {
            AxisDirection::X => 0,
            AxisDirection::Y => 1,
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct MouseMoveAxis(pub AxisDirection);

pub fn binding_part_mouse_move(
    mut binding_parts: Query<(Entity, &MouseMoveAxis, &BindingPartOf, &mut BindingPartData)>,
    mut commands: Commands,
    mut mouse: MessageReader<MouseMotion>,
) {
    for message in mouse.read() {
        for (entity, mouse_move, binding_part_of, mut data) in binding_parts.iter_mut() {
            let value = match mouse_move.0 {
                AxisDirection::X => message.delta.x,
                AxisDirection::Y => message.delta.y,
            };
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
            }
        }
    }
}

#[derive(Debug, Reflect)]
#[reflect(Debug)]
pub enum MouseScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct MouseScroll(pub MouseScrollDirection);

pub fn binding_part_mouse_scroll(
    mut binding_parts: Query<(Entity, &MouseScroll, &BindingPartOf, &mut BindingPartData)>,
    mut commands: Commands,
    mut mouse: MessageReader<MouseWheel>,
) {
    for message in mouse.read() {
        for (entity, mouse_scroll, binding_part_of, mut data) in binding_parts.iter_mut() {
            // Doesn't handle unit :/
            let value = match mouse_scroll.0 {
                MouseScrollDirection::Up => message.y.max(0.0),
                MouseScrollDirection::Down => message.y.min(0.0),
                MouseScrollDirection::Left => message.x.max(0.0),
                MouseScrollDirection::Right => message.x.min(0.0),
            };
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
                // Reset to 0 after triggering
                data.0 = 0.0;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value: 0.0,
                });
            }
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct MouseScrollAxis(pub AxisDirection);

pub fn binding_part_mouse_scroll_axis(
    mut binding_parts: Query<(
        Entity,
        &MouseScrollAxis,
        &BindingPartOf,
        &mut BindingPartData,
    )>,
    mut commands: Commands,
    mut mouse: MessageReader<MouseWheel>,
) {
    for message in mouse.read() {
        for (entity, mouse_scroll_axis, binding_part_of, mut data) in binding_parts.iter_mut() {
            // Doesn't handle unit :/
            let value = match mouse_scroll_axis.0 {
                AxisDirection::X => message.x,
                AxisDirection::Y => message.y,
            };
            if data.0 != value {
                data.0 = value;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value,
                });
                // Reset to 0 after triggering
                data.0 = 0.0;
                commands.trigger(BindingPartUpdate {
                    binding: binding_part_of.0,
                    binding_part: entity,
                    value: 0.0,
                });
            }
        }
    }
}
