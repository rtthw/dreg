<details>
<summary>Table of Contents</summary>

- [Dreg](#dreg)
  - [Features](#features)
  - [License](#license)

</details>

<!-- cargo-rdme start -->

<div align="center">

<br>[![Crate Badge]][Crate] [![Docs Badge]][Docs] [![License Badge]](./LICENSE)

</div>

# Dreg

A simple text-based user interface library that will run on just about anything.

## Features

| Feature                                      | Terminal | Desktop | Web |
|----------------------------------------------|----------|---------|-----|
| Text colors (foreground & background)        | ✅       | ✅      | ✅  |
| Text modifiers (bold, italic, etc.)          | ✅       | ✅*     | ✅* |
| Text layout                                  | ✅       | ✅      | ✅  |
| Keyboard input                               | ✅       | ✅      | ✅  |
| Mouse input                                  | ✅       | ✅      | ✅  |
| Change the window title                      | ✅*      | ✅      | ✅  |
| Custom fonts                                 | ❌       | ✅      | ✅  |
| Change the font at runtime                   | ❌       | ✅      | ✅  |
| Change the font scaling at runtime (zoom in) | ❌       | ✅      | ✅  |
| Render multiple characters to a single cell  | ❌       | ✅      | ✅  |

_*The text modifier type is based on the standard terminal modifiers, and some platforms don't support some of these modifiers._

_*Most terminals support changing the window title. You'd be hard pressed to find one that doesn't._

## License

[MIT](./LICENSE)

[Crate]: https://crates.io/crates/dreg
[Crate Badge]: https://img.shields.io/crates/v/dreg?logo=rust&style=flat-square&logoColor=E05D44&color=E05D44
[Docs Badge]: https://img.shields.io/docsrs/dreg?logo=rust&style=flat-square&logoColor=E05D44
[Docs]: https://docs.rs/dreg
[License Badge]: https://img.shields.io/crates/l/dreg?style=flat-square&color=1370D3
