# Bevy Pretty Nice Input

A refreshingly complex input crate for Bevy.

It works similarly to [bevy-enhanced-input](https://crates.io/crates/bevy_enhanced_input).

An input system entity relates to several `Action` entities, which each relate to several `Binding` entities (which then related to several `BindingPart` entities) and `Condition` entities.

When an input occurs, it may trigger one of several Bevy events:

- `JustPressed<A: Action>`: When an input goes from zero to nonzero.
- `Pressed<A: Action>`: When an input is nonzero.
- `JustReleased<A: Action>`: When an input goes from nonzero to zero.

Actions keep track of previous successful inputs in order to trigger events.
If the previous input is nonexistant, for instance if the input is *invalidated*,
the action is ignored, since the events require a previous input to compare against.

An input system entity can be disabled by inserting `InputDisabled` on it.

## Usage

Add the `PrettyNiceInputPlugin` plugin to your project.

### `input!`

The input system starts with the simple `input!` macro:

```rust
// input!(action, Axis_D[bindings], [conditions])
input!(MyAction, Axis2D[binding2d::wasd()], [Cooldown::new(0.5)])
```

`input!` builds a bundle that can be added to the entity representing the input system or player controller.

### `Action`

`MyAction` is a struct that implements `Action`:

```rust
#[derive(Action)]
#[action(invalidate = true)]
pub struct MyAction;
```

`invalidate` represents what happens when the input system is disabled.
Either the system remembers the previous input state if false, or forgets it if true.
It defaults to `true` if omitted.
`invalidate = false` is best used for actions that need to be maintained while in a certain state.
`invalidate = true` is best used for actions that work regardless of state.

### `Binding`

Bindings can be N-axis, where N is 1 (for single button presses or single joystick axes), 2 (for wasd or entire joysticks), or 3 (composed of other bindings).

Many bindings can be found in the `binding1d` or `binding2d` modules. These are the arbitrary bindings:

- `binding1d::key(key: KeyCode)`: Binding for a single key in the range [0,1].
- `binding1d::key_axis(key_pos: KeyCode, key_neg: KeyCode)`: Binding for two keys in the range [-1,1], with one being positive and the other negative.
- `binding1d::gamepad_axis(axis: GamepadAxis)`: Binding for a single gamepad axis in the range [-1,1].
- `binding1d::mouse_button(button: MouseButton)`: Binding for a single mouse button in the range [0,1].
- `binding1d::mouse_move_axis(axis: AxisDirection)`: Binding for a single axis of mouse movement in the range [-inf,inf].
- `binding1d::mouse_scroll(direction: MouseScrollDirection)`: Binding for a single direction of mouse scroll in the range [0,inf].
- `binding1d::mouse_scroll_axis(axis: AxisDirection)`: Binding for a single axis of mouse scroll in the range [-inf,inf].

There are also shorthand bindings for common scenarios such as `binding1d::space()` and `binding2d::wasd()`.

Bindings may be composed into one bundle:

```rust
input!(... Axis2D[binding2d::wasd()] ...)
// is equivalent to
input!(... Axis2D[(key_axis(KeyCode::KeyD, KeyCode::KeyA), key_axis(KeyCode::KeyW, KeyCode::KeyS))] ...)
```

The bindings given to `input!` must match the dimensionality of the `Axis1D`, `Axis2D`, or `Axis3D` keyword.

### `Condition`

Conditions allow input to be filtered or modified.

The `Condition` trait may be implemented for custom conditions.
The only function is `fn bundle<A: Action>(&self) -> impl Bundle`, which returns a bundle added to the condition entity.

Common conditions include:

- `Cooldown`: Only lets one valid input pass every duration.
- `Filter<F: QueryFilter>`: Only lets the input pass if the query filter matches.
- `InvalidatingFilter<F: QueryFilter>`: Only lets the input pass if the query filter matches. Otherwise, invalidates the input.
- `ButtonPress`: Rising edge filter.
- `ButtonRelease`: Falling edge filter.
- `Invert`: Inverts the update between zero and nonzero, using the last nonzero input when the current input is zero.
- `InputBuffer`: Continues sending nonzero updates for a duration after the input stops being nonzero.
- `ResetBuffer`: Stops any previous input buffers. Doesn't affect the current input in any way.

The conditions array in `input!` may be omitted entirely.