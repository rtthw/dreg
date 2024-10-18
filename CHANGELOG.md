# Changelog

## Unreleased

## 0.2.2 - 10/18/2024

**Core**

- Fixed invalid results for `Rect::hsplit_inverse_len` and `Rect::vsplit_inverse_len`. (#4)

## 0.2.1 - 10/17/2024

**Core**

- Fixed inability to get the last known mouse position.

**Crossterm**

- Fixed cursor display inconsistency.

## 0.2.0 - 10/14/2024

A complete decoupling of Dreg from the terminal backend.

**Terminal**

- Moved to separate `dreg-crossterm` crate.

## 0.1.0 - 8/7/2024

Initial release.
