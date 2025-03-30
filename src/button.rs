/// A virtual gamepad button with a 0 or 1 state.
#[repr(u16)]
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum Button {
    None = 0,
    Up = 1,
    Down = 2,
    Left = 4,
    Right = 8,
    A = 16,
    B = 32,
    X = 64,
    Y = 128,
    Start = 256,
    Select = 512,
    Menu = 1024,
    LeftTrigger = 2048,
    RightTrigger = 4096,
    LeftShoulder = 8192,
    RightShoulder = 16384,
}

#[repr(u16)]
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub enum AnyButton {
    None = 0,
    All = u16::MAX,
    Direction = 1 | 2 | 4 | 8,
    Face = 16 | 32 | 64 | 128,
    System = 256 | 512 | 1024,
    Upper = 2048 | 4096 | 8192 | 16384,
}

impl From<u16> for Button {
    fn from(val: u16) -> Self {
        match val {
            0 => Button::None,
            1 => Button::Up,
            2 => Button::Down,
            4 => Button::Left,
            8 => Button::Right,
            16 => Button::A,
            32 => Button::B,
            64 => Button::X,
            128 => Button::Y,
            256 => Button::Start,
            512 => Button::Select,
            1024 => Button::Menu,
            2048 => Button::LeftTrigger,
            4096 => Button::RightTrigger,
            8192 => Button::LeftShoulder,
            16384 => Button::RightShoulder,
            _ => Button::None,
        }
    }
}

impl From<u16> for AnyButton {
    fn from(val: u16) -> Self {
        match val {
            0 => AnyButton::None,
            15 => AnyButton::Direction,
            240 => AnyButton::Face,
            1792 => AnyButton::System,
            30720 => AnyButton::Upper,
            u16::MAX => AnyButton::All,
            _ => AnyButton::None,
        }
    }
}

impl Button {
    /// Returns the number of variants in the enum
    pub fn len() -> usize {
        16 // Number of variants in the Button enum
    }
}

impl AnyButton {
    /// Returns the number of variants in the enum
    pub fn len() -> usize {
        6 // Number of variants in the Button enum
    }
}
