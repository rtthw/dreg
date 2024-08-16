# Changelog

## Unreleased

- **CHANGE/ADDITION (BREAKING)** - [Kitty image protocol](https://sw.kovidgoyal.net/kitty/graphics-protocol/) support

- **CHANGE/ADDITION (BREAKING)** - Shape elements

## 0.1.0 - 8/7/2024

Initial release.

### Features

- `terminal`

    - `Terminal`

        > An abstraction over the terminal itself providing an interface to [`crossterm`](https://docs.rs/crossterm/0.28.1/crossterm/) and other low-level capabilities.

    - `Frame`

        > A contextual object used by `Program`s to interact with the terminal.

    - `Buffer`

        > The set of `Cell`s to be rendered onto the terminal.

    - `Cell`

        > An individial unit used in the `Buffer`.

    - `Program`

        > The entry point trait for Dreg. The user should create some struct object that implements this trait to interface with the `Frame` and `Terminal`.

    - `Element`

        > The trait used by elements to render onto `Program`s.

- `style`

    - `Style`

        > The way in which `Element`s are presented to the user.

    - `Color`

        > An abstraction over coloring capabilities.

    - `ColorMode`

        > The method `Element`s use to render onto the pre-existing `Cell`s. By default, `ColorMode::Overwrite` simply writes the `Element`'s styling to the `Buffer` as if it was empty.

    - `Modifier`

        > Various changes that terminals support for cell rendering. Things like **bold** and *italic*.

- `primitives`

    - `Pos`

        > Tuple struct for X-Y coordinate space. Values are both `u16`s. A `u8` is just barely too small.
        >
        > Currently, there is no intention of implementing any sort of "size" alternative. It would be exactly the same.

    - `Rect`

        > An abstraction over rectangular portions of the terminal.

- `elements`

    - `Label`

        > An `Element` that renders text onto the `Buffer`.

    - `Block`

        > An `Element` that renders a block to the `Buffer`.

    - `Image`

        > An `Element` that makes use of the [`image-rs`](https://docs.rs/image/latest/image/) crate to render images onto the `Buffer`.
        >
        > Changes needed to be made to the `style` module to account for image transparency. The `ColorMode` object was added to allow the user to define the way in which images are added to the `Buffer`.
        >
        > Currently, only the `Image::Halfblocks` rendering style is supported.

    - `Clear`

        > A `Block` `Element` that simply erases the `Cell`s it covers.
