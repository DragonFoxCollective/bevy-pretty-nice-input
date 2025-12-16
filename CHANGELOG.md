# Changelog

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