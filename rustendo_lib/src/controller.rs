#[derive(Copy, Clone)]
#[repr(u8)]
enum Button {
    A = 0b00000001,
    B = 0b00000010,
    Select = 0b00000100,
    Start = 0b00001000,
    Up = 0b00010000,
    Down = 0b00100000,
    Left = 0b01000000,
    Right = 0b10000000,
}

impl std::ops::Not for Button {
    type Output = u8;

    fn not(self) -> u8 {
        !(self as u8)
    }
}

impl std::ops::BitOr<Button> for u8 {
    type Output = u8;

    fn bitor(self, rhs: Button) -> u8 {
        self | rhs as u8
    }
}

impl std::ops::BitOrAssign<Button> for u8 {
    fn bitor_assign(&mut self, rhs: Button) {
        *self = *self | rhs;
    }
}

pub struct Controller {
    controller: u8,
    latched_controller: u8,
}

impl Controller {
    pub fn new() -> Self {
        Controller {
            controller: 0,
            latched_controller: 0,
        }
    }

    pub fn latch(&mut self) {
        self.latched_controller = self.controller;
    }

    pub fn press_a(&mut self) {
        self.controller |= Button::A;
    }

    pub fn lift_a(&mut self) {
        self.controller &= !Button::A;
    }

    pub fn press_b(&mut self) {
        self.controller |= Button::B;
    }

    pub fn lift_b(&mut self) {
        self.controller &= !Button::B;
    }

    pub fn press_select(&mut self) {
        self.controller |= Button::Select;
    }

    pub fn lift_select(&mut self) {
        self.controller &= !Button::Select;
    }

    pub fn press_start(&mut self) {
        self.controller |= Button::Start;
    }

    pub fn lift_start(&mut self) {
        self.controller &= !Button::Start;
    }

    pub fn press_up(&mut self) {
        self.controller |= Button::Up;
    }

    pub fn lift_up(&mut self) {
        self.controller &= !Button::Up;
    }

    pub fn press_down(&mut self) {
        self.controller |= Button::Down;
    }

    pub fn lift_down(&mut self) {
        self.controller &= !Button::Down;
    }

    pub fn press_left(&mut self) {
        self.controller |= Button::Left;
    }

    pub fn lift_left(&mut self) {
        self.controller &= !Button::Left;
    }

    pub fn press_right(&mut self) {
        self.controller |= Button::Right;
    }

    pub fn lift_right(&mut self) {
        self.controller &= !Button::Right;
    }

    pub fn read_button(&mut self) -> u8 {
        let bit = self.latched_controller & 0x01 == 0x01;
        self.latched_controller >>= 1;
        bit as u8
    }
}
