# Changelog

## 0.6.0

## Added

- Non-unit components can receive the input state directly into them

## Changed

- Reorganized crate. Now there's a proper prelude!

## 0.5.1

### Fixed

- Errors on a case that was ignored

## 0.5.0

- Update to Bevy 0.18

## 0.4.2

### Fixed

- Bindings still exist when input systems despawn

## 0.4.1

### Changed

- Removed dependency on bevy_reflect
- Update README
- Add input_transition! in the example

## 0.4.0

### Changed

- Transitions are now Bundle-based instead of based on single components

### Fixed

- Input buffer sending zeroed data

## 0.3.1

### Changed

- Unidirectional transitions now don't pay attention to the state they move into

### Fixed

- Race conditions in systems

## 0.3.0

### Changed

- Now relies on bevy_event_chain

## 0.2.1

### Added

- Reflection for components

## 0.2.0

### Added

- `ActionData::as_1d_ok`/`as_2d_ok`/`as_3d_ok` which return `Result<_, BevyError>`
- New example for a physics-based (bevy_rapier3D) first-person player controller
- Macro format doc comments

### Changed

- Renamed the crate to use underscores.

## 0.1.0

Initial release!
