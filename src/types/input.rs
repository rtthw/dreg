//! Input Handling



use std::collections::HashSet;



/// A utility object for managing your program's input state.
pub struct InputContext {
    keys_down: HashSet<Scancode>,
    mouse_pos: Option<(u32, u32)>,
    resized: Option<(u16, u16)>,
    newly_focused: bool,
    newly_unfocused: bool,
}

impl Default for InputContext {
    fn default() -> Self {
        Self {
            keys_down: HashSet::new(),
            mouse_pos: None,
            resized: None,
            newly_focused: false,
            newly_unfocused: false,
        }
    }
}

impl InputContext {
    /// **IMPORTANT**: This function must be called at the end of *every* update pass.
    pub fn end_frame(&mut self) {
        self.resized = None;
        self.newly_focused = false;
        self.newly_unfocused = false;
    }

    /// Handle some user [`Input`].
    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::KeyDown(code) => {
                let _repeat = self.handle_key_down(code);
            }
            Input::KeyUp(code) => {
                let _valid_keypress = self.handle_key_up(&code);
            }
            Input::MouseMove(x, y) => {
                self.mouse_pos = Some((x, y));
            }
            Input::Resize(x, y) => {
                self.resized = Some((x, y));
            }
            Input::FocusChange(has_focus) => {
                self.newly_focused = has_focus;
                self.newly_unfocused = !has_focus;
            }
            _ => {}
        }
    }

    /// Shortcut for [`InputContext::handle_input`] with [`Input::KeyDown`],
    /// and the given [`Scancode`].
    pub fn handle_key_down(&mut self, code: Scancode) -> bool {
        self.keys_down.insert(code)
    }

    /// Shortcut for [`InputContext::handle_input`] with [`Input::KeyUp`],
    /// and the given [`Scancode`].
    pub fn handle_key_up(&mut self, code: &Scancode) -> bool {
        self.keys_down.remove(&code)
    }

    /// Get the currently pressed keys.
    ///
    /// Note that this includes *all* [`Scancode`]s, including mouse buttons.
    pub fn keys_down(&self) -> &HashSet<Scancode> {
        &self.keys_down
    }

    /// Get the current mouse position.
    pub fn mouse_pos(&self) -> Option<(u32, u32)> {
        self.mouse_pos
    }

    /// Get the new size for the program's buffer if it was resized this frame.
    pub fn newly_resized_size(&self) -> Option<(u16, u16)> {
        self.resized
    }

    /// Get whether the program's display size has changed this frame.
    pub fn was_resized_this_frame(&self) -> bool {
        self.resized.is_some()
    }
}



#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Input {
    KeyDown(Scancode),
    KeyUp(Scancode),
    MouseMove(u32, u32),

    FocusChange(bool),
    Resize(u16, u16),

    Null,
}



/// A keyboard/mouse button's scancode.
///
/// There are constants for just about every button, and you can still use the extremely uncommon
/// ones with this system. If there isn't explicit support in dreg, then just follow the `evdev`
/// standard (see below).
///
/// A value of `0` is "reserved" (always invalid).
///
/// These values map directly to the `evdev` scancodes for Linux.
/// See https://github.com/emberian/evdev/blob/main/src/scancodes.rs#L26-L572 for reference.
///
/// The keyboard mapping:
///
/// ```text
/// [1]   [59][60][61][62]   [63][64][65][66]   [67][68][87][88]
/// [41][ 2][ 3][ 4][ 5][ 6][ 7][ 8][ 9][10][11][12][13][  14  ]
/// [ 15 ][16][17][18][19][20][21][22][23][24][25][26][27][ 43 ]
/// [  58  ][30][31][32][33][34][35][36][37][38][39][40][  28  ]
/// [   42   ][44][45][46][47][48][49][50][51][52][53][   54   ]
/// [29][125][56][           57           ][100][0x1d0][139][97]
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Scancode(pub u16);

impl From<u16> for Scancode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for Scancode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self.0 {
            0 => "NULL",
            1 => "escape",
            2 => "1",
            3 => "2",
            4 => "3",
            5 => "4",
            6 => "5",
            7 => "6",
            8 => "7",
            9 => "8",
            10 => "9",
            11 => "0",
            12 => "minus",
            13 => "equal",
            14 => "backspace",
            15 => "tab",
            16 => "q",
            17 => "w",
            18 => "e",
            19 => "r",
            20 => "t",
            21 => "y",
            22 => "u",
            23 => "i",
            24 => "o",
            25 => "p",
            26 => "[",
            27 => "]",
            28 => "enter",
            29 => "l_ctrl",
            30 => "a",
            31 => "s",
            32 => "d",
            33 => "f",
            34 => "g",
            35 => "h",
            36 => "j",
            37 => "k",
            38 => "l",
            39 => ";",
            40 => "'",
            41 => "`",
            42 => "l_shift",
            43 => "\\",
            44 => "z",
            45 => "x",
            46 => "c",
            47 => "v",
            48 => "b",
            49 => "n",
            50 => "m",
            51 => ",",
            52 => ".",
            53 => "/",
            54 => "r_shift",
            55 => "kp_asterisk",
            56 => "l_alt",
            57 => "space",
            58 => "capslock",

            102 => "home",
            103 => "up",
            104 => "pageup",
            105 => "left",
            106 => "right",
            107 => "end",
            108 => "down",
            109 => "pagedown",
            110 => "insert",
            111 => "delete",
            _ => "UNKNOWN",
        })
    }
}

// Utilities.
impl Scancode {
    /// Create two [`Scancode`]s from the given character.
    ///
    /// If the character represents something that would normally require the shift key to be
    /// pressed, then the [`Scancode::L_SHIFT`] key is emitted as the first element in the
    /// resulting tuple.
    pub fn from_char(c: char) -> (Option<Self>, Self) {
        match c {
            'a' => (None, Self::A),
            'A' => (Some(Self::L_SHIFT), Self::A),
            'b' => (None, Self::B),
            'B' => (Some(Self::L_SHIFT), Self::B),
            'c' => (None, Self::C),
            'C' => (Some(Self::L_SHIFT), Self::C),
            'd' => (None, Self::D),
            'D' => (Some(Self::L_SHIFT), Self::D),
            'e' => (None, Self::E),
            'E' => (Some(Self::L_SHIFT), Self::E),
            'f' => (None, Self::F),
            'F' => (Some(Self::L_SHIFT), Self::F),
            'g' => (None, Self::G),
            'G' => (Some(Self::L_SHIFT), Self::G),
            'h' => (None, Self::H),
            'H' => (Some(Self::L_SHIFT), Self::H),
            'i' => (None, Self::I),
            'I' => (Some(Self::L_SHIFT), Self::I),
            'j' => (None, Self::J),
            'J' => (Some(Self::L_SHIFT), Self::J),
            'k' => (None, Self::K),
            'K' => (Some(Self::L_SHIFT), Self::K),
            'l' => (None, Self::L),
            'L' => (Some(Self::L_SHIFT), Self::L),
            'm' => (None, Self::M),
            'M' => (Some(Self::L_SHIFT), Self::M),
            'n' => (None, Self::N),
            'N' => (Some(Self::L_SHIFT), Self::N),
            'o' => (None, Self::O),
            'O' => (Some(Self::L_SHIFT), Self::O),
            'p' => (None, Self::P),
            'P' => (Some(Self::L_SHIFT), Self::P),
            'q' => (None, Self::Q),
            'Q' => (Some(Self::L_SHIFT), Self::Q),
            'r' => (None, Self::R),
            'R' => (Some(Self::L_SHIFT), Self::R),
            's' => (None, Self::S),
            'S' => (Some(Self::L_SHIFT), Self::S),
            't' => (None, Self::T),
            'T' => (Some(Self::L_SHIFT), Self::T),
            'u' => (None, Self::U),
            'U' => (Some(Self::L_SHIFT), Self::U),
            'v' => (None, Self::V),
            'V' => (Some(Self::L_SHIFT), Self::V),
            'w' => (None, Self::W),
            'W' => (Some(Self::L_SHIFT), Self::W),
            'x' => (None, Self::X),
            'X' => (Some(Self::L_SHIFT), Self::X),
            'y' => (None, Self::Y),
            'Y' => (Some(Self::L_SHIFT), Self::Y),

            '1' => (None, Self::K_1),
            '!' => (Some(Self::L_SHIFT), Self::K_1),
            '2' => (None, Self::K_2),
            '@' => (Some(Self::L_SHIFT), Self::K_2),
            '3' => (None, Self::K_3),
            '#' => (Some(Self::L_SHIFT), Self::K_3),
            '4' => (None, Self::K_4),
            '$' => (Some(Self::L_SHIFT), Self::K_4),
            '5' => (None, Self::K_5),
            '%' => (Some(Self::L_SHIFT), Self::K_5),
            '6' => (None, Self::K_6),
            '^' => (Some(Self::L_SHIFT), Self::K_6),
            '7' => (None, Self::K_7),
            '&' => (Some(Self::L_SHIFT), Self::K_7),
            '8' => (None, Self::K_8),
            '*' => (Some(Self::L_SHIFT), Self::K_8),
            '9' => (None, Self::K_9),
            '(' => (Some(Self::L_SHIFT), Self::K_9),
            '0' => (None, Self::K_0),
            ')' => (Some(Self::L_SHIFT), Self::K_0),

            '`' => (None, Self::GRAVE),
            '~' => (Some(Self::L_SHIFT), Self::GRAVE),
            '-' => (None, Self::MINUS),
            '_' => (Some(Self::L_SHIFT), Self::MINUS),
            '=' => (None, Self::EQUAL),
            '+' => (Some(Self::L_SHIFT), Self::EQUAL),
            '[' => (None, Self::L_BRACE),
            '{' => (Some(Self::L_SHIFT), Self::L_BRACE),
            ']' => (None, Self::R_BRACE),
            '}' => (Some(Self::L_SHIFT), Self::R_BRACE),
            '\\' => (None, Self::BACKSLASH),
            '|' => (Some(Self::L_SHIFT), Self::BACKSLASH),
            ';' => (None, Self::SEMICOLON),
            ':' => (Some(Self::L_SHIFT), Self::SEMICOLON),
            '\'' => (None, Self::APOSTROPHE),
            '"' => (Some(Self::L_SHIFT), Self::APOSTROPHE),
            '\n' => (None, Self::ENTER), // ???
            ',' => (None, Self::COMMA),
            '<' => (Some(Self::L_SHIFT), Self::COMMA),
            '.' => (None, Self::DOT),
            '>' => (Some(Self::L_SHIFT), Self::DOT),
            '/' => (None, Self::SLASH),
            '?' => (Some(Self::L_SHIFT), Self::SLASH),

            _ => (None, Self(0)),
        }
    }
}

// Constants.
impl Scancode {
    /// A placeholder scancode for when a null code is needed.
    ///
    /// This should always be ignored, as it is always an invalid scancode.
    pub const NULL: Self = Self(0);

    /// The `Escape` key.
    pub const ESC: Self = Self(1);
    /// The `1`/`!` key.
    pub const K_1: Self = Self(2);
    /// The `2`/`@` key.
    pub const K_2: Self = Self(3);
    /// The `3`/`#` key.
    pub const K_3: Self = Self(4);
    /// The `4`/`$` key.
    pub const K_4: Self = Self(5);
    /// The `5`/`%` key.
    pub const K_5: Self = Self(6);
    /// The `6`/`^` key.
    pub const K_6: Self = Self(7);
    /// The `7`/`&` key.
    pub const K_7: Self = Self(8);
    /// The `8`/`*` key.
    pub const K_8: Self = Self(9);
    /// The `9`/`(` key.
    pub const K_9: Self = Self(10);
    /// The `0`/`)` key.
    pub const K_0: Self = Self(11);
    /// The `-`/`_` key.
    pub const MINUS: Self = Self(12);
    /// The `=`/`+` key.
    pub const EQUAL: Self = Self(13);
    /// The `Backspace` key.
    pub const BACKSPACE: Self = Self(14);
    /// The `Tab` key.
    pub const TAB: Self = Self(15);
    /// The `q` key.
    pub const Q: Self = Self(16);
    /// The `w` key.
    pub const W: Self = Self(17);
    /// The `e` key.
    pub const E: Self = Self(18);
    /// The `r` key.
    pub const R: Self = Self(19);
    /// The `t` key.
    pub const T: Self = Self(20);
    /// The `y` key.
    pub const Y: Self = Self(21);
    /// The `u` key.
    pub const U: Self = Self(22);
    /// The `i` key.
    pub const I: Self = Self(23);
    /// The `o` key.
    pub const O: Self = Self(24);
    /// The `p` key.
    pub const P: Self = Self(25);
    /// The `[`/`{` key.
    pub const L_BRACE: Self = Self(26);
    /// The `]`/`}` key.
    pub const R_BRACE: Self = Self(27);
    /// The `Enter`/`Return` key.
    pub const ENTER: Self = Self(28);
    /// The left `Control`/`Command` key.
    pub const L_CTRL: Self = Self(29);
    /// The `a` key.
    pub const A: Self = Self(30);
    /// The `s` key.
    pub const S: Self = Self(31);
    /// The `d` key.
    pub const D: Self = Self(32);
    /// The `f` key.
    pub const F: Self = Self(33);
    /// The `g` key.
    pub const G: Self = Self(34);
    /// The `h` key.
    pub const H: Self = Self(35);
    /// The `i` key.
    pub const J: Self = Self(36);
    /// The `k` key.
    pub const K: Self = Self(37);
    /// The `l` key.
    pub const L: Self = Self(38);
    /// The `;`/`:` key.
    pub const SEMICOLON: Self = Self(39);
    /// The `'`/`"` key.
    pub const APOSTROPHE: Self = Self(40);
    /// The ```/`~` key.
    pub const GRAVE: Self = Self(41);
    /// The left `Shift` key.
    pub const L_SHIFT: Self = Self(42);
    /// The `\`/`|` key.
    pub const BACKSLASH: Self = Self(43);
    /// The `z` key.
    pub const Z: Self = Self(44);
    /// The `x` key.
    pub const X: Self = Self(45);
    /// The `c` key.
    pub const C: Self = Self(46);
    /// The `v` key.
    pub const V: Self = Self(47);
    /// The `b` key.
    pub const B: Self = Self(48);
    /// The `n` key.
    pub const N: Self = Self(49);
    /// The `m` key.
    pub const M: Self = Self(50);
    /// The `,`/`<` key.
    pub const COMMA: Self = Self(51);
    /// The `.`/`>` key.
    pub const DOT: Self = Self(52);
    /// The `/`/`?` key.
    pub const SLASH: Self = Self(53);
    /// The right `Shift` key.
    pub const R_SHIFT: Self = Self(54);
    /// The left `Alt`/`Option` key.
    pub const L_ALT: Self = Self(56);
    /// The `Space` bar key.
    pub const SPACE: Self = Self(57);
    /// The `Capslock` key.
    pub const CAPSLOCK: Self = Self(58);

    pub const F1: Self = Self(59);
    pub const F2: Self = Self(60);
    pub const F3: Self = Self(61);
    pub const F4: Self = Self(62);
    pub const F5: Self = Self(63);
    pub const F6: Self = Self(64);
    pub const F7: Self = Self(65);
    pub const F8: Self = Self(66);
    pub const F9: Self = Self(67);
    pub const F10: Self = Self(68);

    pub const NUMLOCK: Self = Self(69);
    pub const SCROLLLOCK: Self = Self(70);

    pub const KP_ASTERISK: Self = Self(55);
    pub const KP_7: Self = Self(71);
    pub const KP_8: Self = Self(72);
    pub const KP_9: Self = Self(73);
    pub const KP_MINUS: Self = Self(74);
    pub const KP_4: Self = Self(75);
    pub const KP_5: Self = Self(76);
    pub const KP_6: Self = Self(77);
    pub const KP_PLUS: Self = Self(78);
    pub const KP_1: Self = Self(79);
    pub const KP_2: Self = Self(80);
    pub const KP_3: Self = Self(81);
    pub const KP_0: Self = Self(82);
    pub const KP_DOT: Self = Self(83);

    /// The right `Control`/`Command` key.
    pub const R_CTRL: Self = Self(97);
    /// The right `Alt`/`Option` key.
    pub const R_ALT: Self = Self(100);

    /// The `Home` key.
    pub const HOME: Self = Self(102);
    /// The `Arrow Up` key.
    pub const UP: Self = Self(103);
    /// The `Page Up` key.
    pub const PAGEUP: Self = Self(104);
    /// The `Arrow Left` key.
    pub const LEFT: Self = Self(105);
    /// The `Arrow Right` key.
    pub const RIGHT: Self = Self(106);
    /// The `End` key.
    pub const END: Self = Self(107);
    /// The `Arrow Down` key.
    pub const DOWN: Self = Self(108);
    /// The `Page Down` key.
    pub const PAGEDOWN: Self = Self(109);
    /// The `Insert` key.
    pub const INSERT: Self = Self(110);
    /// The `Delete` key.
    pub const DELETE: Self = Self(111);

    /// The `Wheel Up` mouse button.
    pub const SCROLLUP: Self = Self(177);
    /// The `Wheel Down` mouse button.
    pub const SCROLLDOWN: Self = Self(178);
    /// The `Left` mouse button.
    pub const LMB: Self = Self(0x110);
    /// The `Right` mouse button.
    pub const RMB: Self = Self(0x111);
    /// The `Middle` mouse button (middle click).
    pub const MMB: Self = Self(0x112);
    /// The `Forward` mouse button.
    pub const MOUSE_FORWARD: Self = Self(0x115);
    /// The `Back` mouse button.
    pub const MOUSE_BACK: Self = Self(0x116);
}
