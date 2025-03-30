/// A virtual gamepad button with a 0 or 1 state.
#[repr(u16)]
#[derive(Clone, Copy, Debug)]
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
    Any = u16::MAX,
    AnyDirection = 1 | 2 | 4 | 8,
    AnyFace = 16 | 32 | 64 | 128,
    AnySystem = 256 | 512 | 1024,
    AnyUpper = 2048 | 4096 | 8192 | 16384,
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
            15 => Button::AnyDirection,
            240 => Button::AnyFace,
            1792 => Button::AnySystem,
            30720 => Button::AnyUpper,
            u16::MAX => Button::Any,
            _ => Button::None,
        }
    }
}
