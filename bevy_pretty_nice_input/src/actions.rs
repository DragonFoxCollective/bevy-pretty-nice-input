use std::marker::PhantomData;

use bevy::prelude::*;

use crate::conditions::Condition;

#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Clone, Debug, PartialEq)]
pub enum ActionData {
    Axis1D(f32),
    Axis2D(Vec2),
    Axis3D(Vec3),
}

impl ActionData {
    pub fn x(x: f32) -> Self {
        ActionData::Axis1D(x)
    }

    pub fn xy(x: f32, y: f32) -> Self {
        ActionData::Axis2D(Vec2::new(x, y))
    }

    pub fn xyz(x: f32, y: f32, z: f32) -> Self {
        ActionData::Axis3D(Vec3::new(x, y, z))
    }
}

impl ActionData {
    pub fn as_1d(&self) -> Option<f32> {
        if let ActionData::Axis1D(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_1d_ok(&self) -> Result<f32> {
        match self {
            ActionData::Axis1D(value) => Ok(*value),
            ActionData::Axis2D(_) => Err(BevyError::from("Expected Axis1D, found Axis2D")),
            ActionData::Axis3D(_) => Err(BevyError::from("Expected Axis1D, found Axis3D")),
        }
    }

    pub fn as_2d(&self) -> Option<Vec2> {
        if let ActionData::Axis2D(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_2d_ok(&self) -> Result<Vec2> {
        match self {
            ActionData::Axis1D(_) => Err(BevyError::from("Expected Axis2D, found Axis1D")),
            ActionData::Axis2D(value) => Ok(*value),
            ActionData::Axis3D(_) => Err(BevyError::from("Expected Axis2D, found Axis3D")),
        }
    }

    pub fn as_3d(&self) -> Option<Vec3> {
        if let ActionData::Axis3D(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    pub fn as_3d_ok(&self) -> Result<Vec3> {
        match self {
            ActionData::Axis1D(_) => Err(BevyError::from("Expected Axis3D, found Axis1D")),
            ActionData::Axis2D(_) => Err(BevyError::from("Expected Axis3D, found Axis2D")),
            ActionData::Axis3D(value) => Ok(*value),
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            ActionData::Axis1D(value) => *value == 0.0,
            ActionData::Axis2D(value) => *value == Vec2::ZERO,
            ActionData::Axis3D(value) => *value == Vec3::ZERO,
        }
    }

    pub fn zeroed(&self) -> Self {
        match self {
            ActionData::Axis1D(_) => ActionData::Axis1D(0.0),
            ActionData::Axis2D(_) => ActionData::Axis2D(Vec2::ZERO),
            ActionData::Axis3D(_) => ActionData::Axis3D(Vec3::ZERO),
        }
    }

    pub fn length(&self) -> f32 {
        match self {
            ActionData::Axis1D(value) => value.abs(),
            ActionData::Axis2D(value) => value.length(),
            ActionData::Axis3D(value) => value.length(),
        }
    }

    pub fn is_pressed_with(&self, threshold: f32) -> bool {
        self.length() > threshold
    }

    pub fn debug_name(&self) -> &'static str {
        match self {
            ActionData::Axis1D(_) => "Axis1D",
            ActionData::Axis2D(_) => "Axis2D",
            ActionData::Axis3D(_) => "Axis3D",
        }
    }
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component, Debug)]
pub struct PrevActionData(pub ActionData);

#[derive(Component, Default, Debug, Reflect)]
#[reflect(Component, Default, Debug)]
pub struct PrevAction2Data(pub Option<ActionData>);

pub trait Action: Send + Sync + 'static {
    /// Which filter determines how enabled/disabled input is processed. Generally, this should either be [`IsInputEnabled`] or [`IsInputEnabledInvalidate`].
    type EnableFilter: Condition;
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = ActionOf<A>, linked_spawn)]
pub struct Actions<A: Action>(#[relationship] Vec<Entity>, PhantomData<A>);

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = Actions<A>)]
pub struct ActionOf<A: Action>(#[relationship] pub Entity, PhantomData<A>);
