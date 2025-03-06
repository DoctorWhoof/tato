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
    #[inline(always)]
    pub fn copy_current_to_previous_state(&mut self) {
        self.previous = self.state;
    }

    /// Whether the given button is currentl down.
    #[inline(always)]
    pub fn is_down(&self, button: Button) -> bool {
        (self.state & button as u16) != 0
    }

    /// Whether the given button is currentl up.
    #[inline(always)]
    pub fn is_up(&self, button: Button) -> bool {
        (self.state & button as u16) == 0
    }

    /// Whether the given button has just been pressed this frame. Requires
    /// [copy_current_to_previous_state] to have been called at the beginning of the frame
    #[inline(always)]
    pub fn is_just_pressed(&self, button: Button) -> bool {
        self.is_down(button) && (self.previous & button as u16 == 0)
    }

    /// Whether the given button has just been released this frame. Requires
    /// [copy_current_to_previous_state] to have been called at the beginning of the frame
    #[inline(always)]
    pub fn is_just_released(&self, button: Button) -> bool {
        !self.is_down(button) && (self.previous & button as u16 != 0)
    }

    /// A single u16 where each bit represents a button pressed or not.
    #[inline(always)]
    pub fn buttons(&self) -> u16 {
        self.state
    }

    /// Sets the bit for a particular button.
    #[inline(always)]
    pub fn set_state(&mut self, button:Button, value:bool){
        if value {
            self.state |= button as u16;
        } else {
            self.state &= !(button as u16);
        }
    }
}
