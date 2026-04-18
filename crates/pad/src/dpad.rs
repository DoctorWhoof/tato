use crate::{AnyButton, Button};

/// A simple virtual Game controller with only digital buttons.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DPad {
    pub state: u16,
    pub previous: u16,
    pub allow_diagonals: bool,
}

impl Default for DPad {
    fn default() -> Self {
        Self { state: 0, previous: 0, allow_diagonals: true }
    }
}

impl DPad {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default()
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
    /// Dpad::copy_current_to_previous_state() to have been called at the beginning of the frame
    #[inline(always)]
    pub fn is_just_pressed(&self, button: Button) -> bool {
        self.is_down(button) && (self.previous & button as u16 == 0)
    }

    /// Whether the given button has just been released this frame. Requires
    /// Dpad::copy_current_to_previous_state() to have been called at the beginning of the frame
    #[inline(always)]
    pub fn is_just_released(&self, button: Button) -> bool {
        !self.is_down(button) && (self.previous & button as u16 != 0)
    }

    /// Whether any button in the group is currently down.
    #[inline(always)]
    pub fn is_any_down(&self, button_group: AnyButton) -> bool {
        (self.state & button_group as u16) != 0
    }

    /// Whether any button in the group was just pressed this frame.
    #[inline(always)]
    pub fn is_any_just_pressed(&self, button_group: AnyButton) -> bool {
        let current_state = self.state & button_group as u16;
        let previous_state = self.previous & button_group as u16;

        current_state != 0 && previous_state == 0
    }

    /// Whether any button in the group was just released this frame.
    #[inline(always)]
    pub fn is_any_just_released(&self, button_group: AnyButton) -> bool {
        let current_state = self.state & button_group as u16;
        let previous_state = self.previous & button_group as u16;

        current_state == 0 && previous_state != 0
    }

    /// Sets the bit for a particular button.
    #[inline(always)]
    pub fn set_state(&mut self, button: Button, value: bool) {
        if value && !self.allow_diagonals {
            let v_mask: u16 = Button::Up as u16 | Button::Down as u16;
            let h_mask: u16 = Button::Left as u16 | Button::Right as u16;
            let btn = button as u16;

            if (btn & v_mask) != 0 && (self.state & h_mask) != 0 {
                // Setting vertical while horizontal is active.
                // Vertical only wins if it was already held last frame and horizontal wasn't.
                if (self.previous & v_mask) != 0 && (self.previous & h_mask) == 0 {
                    self.state &= !h_mask;
                } else {
                    return; // horizontal holds, or tiebreak favors horizontal
                }
            } else if (btn & h_mask) != 0 && (self.state & v_mask) != 0 {
                // Setting horizontal while vertical is active.
                // Vertical only holds if it was already held last frame and horizontal wasn't.
                if (self.previous & v_mask) != 0 && (self.previous & h_mask) == 0 {
                    return; // vertical holds
                } else {
                    self.state &= !v_mask; // horizontal wins, or tiebreak favors horizontal
                }
            }
        }

        if value {
            self.state |= button as u16;
        } else {
            self.state &= !(button as u16);
        }
    }
}
