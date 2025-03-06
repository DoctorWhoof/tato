use crate::Button;

/// A simple virtual Game controller with only digital buttons.
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct DPad{
    state:u16,
    previous:u16
}

impl DPad {
    pub fn new() -> Self {
        Self::default()
    }

    /// Must be called once per frame, prior to setting any value for the current state.
    pub fn copy_current_to_previous_state(&mut self){
        self.previous = self.state;
    }

    pub fn is_pressed(&self, button:Button) -> bool{
        (self.state & button as u16) != 0
    }

    pub fn is_released(&self, button:Button) -> bool{
        (self.state & button as u16) == 0
    }

    pub fn is_just_pressed(&self, button:Button) -> bool{
        self.is_pressed(button) && (self.previous & button as u16 == 0)
    }

    pub fn is_just_released(&self, button:Button) -> bool{
        !self.is_pressed(button) && (self.previous & button as u16 != 0)
    }

    pub fn state(&self) -> u16 {
        self.state
    }

    /// Sets the bit for a particular button.
    pub fn set_state(&mut self, button:Button, value:bool){
        if value {
            self.state |= button as u16;
        } else {
            self.state &= !(button as u16);
        }
    }
}
