
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
   LeftTrigger = 1024,
   RightTrigger = 2048,
   LeftShoulder = 4096,
   RightShoulder = 8192,
   Menu = 16384,
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
           1024 => Button::LeftTrigger,
           2048 => Button::RightTrigger,
           4096 => Button::LeftShoulder,
           8192 => Button::RightShoulder,
           16384 => Button::Menu,
           _ => Button::None
       }
   }
}
