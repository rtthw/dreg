<details>
<summary>Table of Contents</summary>

- [Dreg](#dreg)
  - [Quickstart](#quickstart)
  - [Overview](#overview)
    - [Design Philosophy](#design-philosophy)
    - [Limitations](#limitations)
  - [Acknowledgments](#acknowledgments)
  - [License](#license)

</details>

<!-- cargo-rdme start -->

<div align="center">

<br>[![Crate Badge]][Crate] [![Docs Badge]][Docs] [![License Badge]](./LICENSE)

</div>

# Dreg

A simple text-based user interface library that will run on just about anything.

## Quickstart

```rust
```

## Overview

### Design Philosophy

The design of Dreg has been radical simplicity from the very start.

### Features

- Runs on anything[^1].

### Limitations

- No support for variable width fonts; even on platforms that do support them.

## Acknowledgments

- [`ratatui`] & [`tui-rs`], for the original inspiration for the project.

## License

[MIT](./LICENSE)

[`ratatui`]: https://docs.rs/ratatui/latest/ratatui/
[`tui-rs`]: https://docs.rs/tui/latest/tui/
[Crate]: https://crates.io/crates/dreg
[Crate Badge]: https://img.shields.io/crates/v/dreg?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44
[Docs Badge]: https://img.shields.io/docsrs/dreg?logo=rust&style=flat-square&logoColor=E05D44
[Docs]: https://docs.rs/dreg
[License Badge]: https://img.shields.io/crates/l/dreg?style=flat-square&color=1370D3

[^1]: It's not gonna run on your toaster, but it might run on your smart fridge.
