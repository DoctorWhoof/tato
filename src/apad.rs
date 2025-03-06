use crate::{Button, DPad};

/// A simple virtual Game controller with digital buttons and a few analogue axis.
#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct APad {
    pub left_stick_x: i16,
    pub left_stick_y: i16,
    pub buttons: DPad,
}

impl APad {
    pub fn new() -> Self {
        Self::default()
    }

    /// Must be called once per frame, prior to setting any value for the current state.
    #[inline(always)]
    pub fn copy_current_to_previous_state(&mut self) {
        self.buttons.copy_current_to_previous_state();
    }

    /// Whether the given button is currentl down.
    #[inline(always)]
    pub fn is_down(&self, button: Button) -> bool {
        self.buttons.is_down(button)
    }

    /// Whether the given button is currentl up.
    #[inline(always)]
    pub fn is_up(&self, button: Button) -> bool {
        self.buttons.is_up(button)
    }

    /// Whether the given button has just been pressed this frame. Requires
    /// [copy_current_to_previous_state] to have been called at the beginning of the frame
    #[inline(always)]
    pub fn is_just_pressed(&self, button: Button) -> bool {
        self.buttons.is_just_pressed(button)
    }

    /// Whether the given button has just been released this frame. Requires
    /// [copy_current_to_previous_state] to have been called at the beginning of the frame
    #[inline(always)]
    pub fn is_just_released(&self, button: Button) -> bool {
        self.buttons.is_just_released(button)
    }

    /// A single u16 where each bit represents a button pressed or not.
    #[inline(always)]
    pub fn buttons(&self) -> u16 {
        self.buttons.buttons()
    }

    /// The X state of the left stick in the -1.0 to 1.0 range
    #[inline(always)]
    pub fn left_stick_x(&self) -> f32 {
        self.left_stick_x as f32 / i16::MAX as f32
    }

    /// The Y state of the left stick in the -1.0 to 1.0 range
    #[inline(always)]
    pub fn left_stick_y(&self) -> f32 {
        self.left_stick_y as f32 / i16::MAX as f32
    }

    /// Sets the bit for a particular button.
    #[inline(always)]
    pub fn set_button(&mut self, button: Button, value: bool) {
        self.buttons.set_state(button, value);
    }

    /// Converts from f32 to i16, clamps anything outside -1.0 to 1.0 range.
    #[inline(always)]
    pub fn set_left_stick_x(&mut self, x: f32) {
        self.left_stick_x = (x.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
    }

    /// Converts from f32 to i16, clamps anything outside -1.0 to 1.0 range.
    #[inline(always)]
    pub fn set_left_stick_y(&mut self, y: f32) {
        self.left_stick_y = (y.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
    }
}
