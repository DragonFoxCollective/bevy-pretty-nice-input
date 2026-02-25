use std::marker::PhantomData;

use bevy::prelude::*;

use crate::actions::{Action, ActionData};

/// Event fired when an input goes from zero to nonzero.
#[derive(EntityEvent, Debug, Reflect)]
pub struct JustPressed<A: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<A>,
}

impl<A: Action> Clone for JustPressed<A> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            data: self.data,
            _marker: PhantomData,
        }
    }
}

/// Event fired when an input is nonzero.
#[derive(EntityEvent, Debug, Reflect)]
pub struct Pressed<A: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<A>,
}

impl<A: Action> Clone for Pressed<A> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            data: self.data,
            _marker: PhantomData,
        }
    }
}

/// Event fired when an input goes from nonzero to zero.
#[derive(EntityEvent, Debug, Reflect)]
pub struct JustReleased<A: Action> {
    #[event_target]
    pub input: Entity,
    pub _marker: PhantomData<A>,
}

impl<A: Action> Clone for JustReleased<A> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            _marker: PhantomData,
        }
    }
}

/// Event fired when input happens at all.
#[derive(EntityEvent, Debug, Reflect)]
pub struct Updated<A: Action> {
    #[event_target]
    pub input: Entity,
    pub data: ActionData,
    pub _marker: PhantomData<A>,
}

impl<A: Action> Clone for Updated<A> {
    fn clone(&self) -> Self {
        Self {
            input: self.input,
            data: self.data,
            _marker: PhantomData,
        }
    }
}
