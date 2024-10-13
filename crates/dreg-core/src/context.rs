

use std::collections::HashSet;



pub struct Context {
    keys_down: HashSet<Scancode>,
    last_mouse_pos: Option<(u16, u16)>,
}

impl Context {
    pub fn handle_input(&mut self, input: Input) {
        match input {
            Input::KeyDown(code) => {
                let _repeat = self.handle_key_down(code);
            }
            Input::KeyUp(code) => {
                let _valid = self.handle_key_up(&code);
            }
            Input::MouseMove { x, y } => {
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
    /// Left mouse button.
    pub const LMB: Self = Self(0x110);
    /// Right mouse button.
    pub const RMB: Self = Self(0x111);
    /// Middle mouse button (middle click).
    pub const MMB: Self = Self(0x112);
}


#[derive(Clone, Copy)]
pub enum Input {
    KeyDown(Scancode),
    KeyUp(Scancode),
    MouseMove { x: u16, y: u16 },

    Null,
}
