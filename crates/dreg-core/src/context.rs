

use std::collections::HashSet;



pub struct Context {
    keys_down: HashSet<Scancode>,
    last_mouse_pos: Option<(u16, u16)>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            keys_down: HashSet::new(),
            last_mouse_pos: None,
        }
    }
}

impl Context {
    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::KeyDown(code) => {
                let _repeat = self.handle_key_down(code);
            }
            Input::KeyUp(code) => {
                let _valid_keypress = self.handle_key_up(&code);
            }
            Input::MouseMove(x, y) => {
                self.last_mouse_pos = Some((x, y));
            }
            _ => {}
        }
    }

    pub fn handle_key_down(&mut self, code: Scancode) -> bool {
        self.keys_down.insert(code)
    }

    pub fn handle_key_up(&mut self, code: &Scancode) -> bool {
        self.keys_down.remove(&code)
    }
}



#[derive(Clone, Copy)]
pub enum Input {
    KeyDown(Scancode),
    KeyUp(Scancode),
    MouseMove(u16, u16),

    FocusChange(bool),
    Resize(u16, u16),

    Null,
}


/// A button's scancode. Maps directly to the `evdev` scancodes.
///
/// A value of `0` is "reserved" (always invalid).
///
/// See https://github.com/emberian/evdev/blob/main/src/scancodes.rs#L26-L572 for reference.
/// ```
/// [1]   [59][60][61][62]   [63][64][65][66]   [67][68][87][88]
/// [41][ 2][ 3][ 4][ 5][ 6][ 7][ 8][ 9][10][11][12][13][  14  ]
/// [ 15 ][16][17][18][19][20][21][22][23][24][25][26][27][ 43 ]
/// [  58  ][30][31][32][33][34][35][36][37][38][39][40][  28  ]
/// [   42   ][44][45][46][47][48][49][50][52][52][53][   54   ]
/// [29][125][56][           57           ][100][0x1d0][139][97]
/// ```
#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Scancode(pub u16);

impl From<u16> for Scancode {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl Scancode {
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

impl Scancode {
    pub const NULL: Self = Self(0);

    pub const ESC: Self = Self(1);
    pub const K_1: Self = Self(2);
    pub const K_2: Self = Self(3);
    pub const K_3: Self = Self(4);
    pub const K_4: Self = Self(5);
    pub const K_5: Self = Self(6);
    pub const K_6: Self = Self(7);
    pub const K_7: Self = Self(8);
    pub const K_8: Self = Self(9);
    pub const K_9: Self = Self(10);
    pub const K_0: Self = Self(11);
    pub const MINUS: Self = Self(12);
    pub const EQUAL: Self = Self(13);
    pub const BACKSPACE: Self = Self(14);
    pub const TAB: Self = Self(15);
    pub const Q: Self = Self(16);
    pub const W: Self = Self(17);
    pub const E: Self = Self(18);
    pub const R: Self = Self(19);
    pub const T: Self = Self(20);
    pub const Y: Self = Self(21);
    pub const U: Self = Self(22);
    pub const I: Self = Self(23);
    pub const O: Self = Self(24);
    pub const P: Self = Self(25);
    pub const L_BRACE: Self = Self(26);
    pub const R_BRACE: Self = Self(27);
    pub const ENTER: Self = Self(28);
    pub const L_CTRL: Self = Self(29);
    pub const A: Self = Self(30);
    pub const S: Self = Self(31);
    pub const D: Self = Self(32);
    pub const F: Self = Self(33);
    pub const G: Self = Self(34);
    pub const H: Self = Self(35);
    pub const J: Self = Self(36);
    pub const K: Self = Self(37);
    pub const L: Self = Self(38);
    pub const SEMICOLON: Self = Self(39);
    pub const APOSTROPHE: Self = Self(40);
    pub const GRAVE: Self = Self(41);
    pub const L_SHIFT: Self = Self(42);
    pub const BACKSLASH: Self = Self(43);
    pub const Z: Self = Self(44);
    pub const X: Self = Self(45);
    pub const C: Self = Self(46);
    pub const V: Self = Self(47);
    pub const B: Self = Self(48);
    pub const N: Self = Self(49);
    pub const M: Self = Self(50);
    pub const COMMA: Self = Self(51);
    pub const DOT: Self = Self(52);
    pub const SLASH: Self = Self(53);
    pub const R_SHIFT: Self = Self(54);
    pub const KP_ASTERISK: Self = Self(55);
    pub const L_ALT: Self = Self(56);
    pub const SPACE: Self = Self(57);
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

    pub const R_CTRL: Self = Self(97);
    pub const R_ALT: Self = Self(100);

    pub const HOME: Self = Self(102);
    pub const UP: Self = Self(103);
    pub const PAGEUP: Self = Self(104);
    pub const LEFT: Self = Self(105);
    pub const RIGHT: Self = Self(106);
    pub const END: Self = Self(107);
    pub const DOWN: Self = Self(108);
    pub const PAGEDOWN: Self = Self(109);
    pub const INSERT: Self = Self(110);
    pub const DELETE: Self = Self(111);
}
