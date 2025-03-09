#![no_std]

// TODO: Determine how much rects are overlapping if Layout is too small, and shrink each one accordingly
// OR return result with difference
//
// TODO: Distinction between margin and gap
// TODO: Vec2

/// Represents a rectangle with position and dimensions.
///
/// A rectangle is defined by its top-left corner coordinates (x, y)
/// and its width and height.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    /// X-coordinate of the top-left corner
    pub x: u16,
    /// Y-coordinate of the top-left corner
    pub y: u16,
    /// Width of the rectangle
    pub w: u16,
    /// Height of the rectangle
    pub h: u16,
}

/// A layout frame that manages rectangular areas with margins and scaling.
///
/// A frame consists of an outer rectangle, an inner cursor rectangle (available space),
/// and properties that control how child frames are created and positioned.
#[derive(Debug, Clone)]
pub struct Frame {
    /// The outer rectangle defining the frame boundaries
    pub rect: Rect,
    /// Inner rectangle representing available space
    cursor: Rect,
    /// Scaling factor for dimensions
    scale: f32,
    /// Margin size between frames
    margin: u16,
}

/// Represents the side of a frame where a child frame can be added.
#[derive(Debug, Clone, Copy)]
enum Side {
    /// Left side of the frame
    Left,
    /// Right side of the frame
    Right,
    /// Top side of the frame
    Top,
    /// Bottom side of the frame
    Bottom,
}

impl Frame {
    /// Creates a new frame with the specified outer rectangle.
    /// Initializes with default values for scale (1.0) and margin (5 pixels).
    pub fn new(rect: Rect) -> Self {
        let scale = 1.0;
        let margin = 5;
        let cursor = rect_shrink(rect, margin);
        Self {
            rect,
            cursor,
            margin,
            scale,
        }
    }

    /// Sets a new margin value and recalculates the cursor rectangle.
    pub fn set_margin(&mut self, margin: u16) {
        //remove old margin
        self.cursor = rect_expand(self.cursor, self.margin);
        // apply new margin
        self.margin = margin;
        self.cursor = rect_shrink(self.rect, self.margin);
    }

    /// Returns the current margin value.
    pub fn margin(&self) -> u16 {
        self.margin
    }

    /// Sets a new scale factor for the frame.
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        // self.set_margin(self.margin);
    }

    /// Returns the current scale factor.
    pub fn scale(&self) -> f32 {
        self.scale
    }


    fn add(&mut self, side: Side, len: u16, scale: f32, mut func: impl FnMut(&mut Frame)) {
        let scaled_len = (len as f32 * scale) as u16;
        let margin = (self.margin as f32 * self.scale) as u16;

        // Calculate the child rectangle based on the side
        let child_rect = match side {
            Side::Left => Rect {
                x: self.cursor.x,
                y: self.cursor.y,
                w: scaled_len,
                h: self.cursor.h,
            },
            Side::Right => Rect {
                x: (self.cursor.x + self.cursor.w).saturating_sub(scaled_len),
                y: self.cursor.y,
                w: scaled_len,
                h: self.cursor.h,
            },
            Side::Top => Rect {
                x: self.cursor.x,
                y: self.cursor.y,
                w: self.cursor.w,
                h: scaled_len,
            },
            Side::Bottom => Rect {
                x: self.cursor.x,
                y: (self.cursor.y + self.cursor.h).saturating_sub(scaled_len),
                w: self.cursor.w,
                h: scaled_len,
            },
        };

        let child_cursor = rect_shrink(child_rect, margin);

        // Check if the child fits
        let is_horizontal = matches!(side, Side::Left | Side::Right);
        let dimension = if is_horizontal {
            child_rect.w
        } else {
            child_rect.h
        };
        let parent_dimension = if is_horizontal {
            self.cursor.w
        } else {
            self.cursor.h
        };
        if dimension > parent_dimension || dimension < margin {
            return;
        }

        // Update parent cursor
        match side {
            Side::Left => {
                self.cursor.x += scaled_len + margin;
                self.cursor.w = self.cursor.w.saturating_sub(scaled_len + margin);
            }
            Side::Right => {
                self.cursor.w = self.cursor.w.saturating_sub(scaled_len + margin);
            }
            Side::Top => {
                self.cursor.y += scaled_len + margin;
                self.cursor.h = self.cursor.h.saturating_sub(scaled_len + margin);
            }
            Side::Bottom => {
                self.cursor.h = self.cursor.h.saturating_sub(scaled_len + margin);
            }
        }

        // Call the function with the new frame
        func(&mut Frame {
            rect: child_rect,
            cursor: child_cursor,
            margin: self.margin,
            scale: self.scale,
        })
    }

    /// Adds a new frame on the left side with specified width.
    /// # Parameters
    /// * `len` - Width of the new frame (before scaling)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn add_left(&mut self, len: u16, func: impl FnMut(&mut Frame)) {
        self.add(Side::Left, len, self.scale, func)
    }

    /// Adds a new frame on the right side with specified width.
    /// # Parameters
    /// * `len` - Width of the new frame (before scaling)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn add_right(&mut self, len: u16, func: impl FnMut(&mut Frame)) {
        self.add(Side::Right, len, self.scale, func)
    }

    /// Adds a new frame on the top with specified height.
    /// # Parameters
    /// * `len` - Height of the new frame (before scaling)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn add_top(&mut self, len: u16, func: impl FnMut(&mut Frame)) {
        self.add(Side::Top, len, self.scale, func)
    }

    /// Adds a new frame on the bottom with specified height.
    /// # Parameters
    /// * `len` - Height of the new frame (before scaling)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn add_bottom(&mut self, len: u16, func: impl FnMut(&mut Frame)) {
        self.add(Side::Bottom, len, self.scale, func)
    }

    /// Creates a frame on the specified side taking a proportion of available space.
    /// This is an internal method used by the public fill_* methods.
    /// # Parameters
    /// * `side` - Which side to add the child frame to
    /// * `ratio` - Proportion of available space (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    fn fill(&mut self, side: Side, ratio: f32, func: impl FnMut(&mut Frame)) {
        let is_horizontal = matches!(side, Side::Left | Side::Right);
        let len = if is_horizontal {
            self.cursor.w as f32 * ratio.clamp(0.0, 1.0)
        } else {
            self.cursor.h as f32 * ratio.clamp(0.0, 1.0)
        };

        self.add(side, len as u16, 1.0, func);
    }

    /// Creates a frame on the left taking a proportion of available width.
    ///
    /// # Parameters
    /// * `ratio` - Proportion of available width (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn fill_left(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        self.fill(Side::Left, ratio, func);
    }

    /// Creates a frame on the top taking a proportion of available height.
    /// # Parameters
    /// * `ratio` - Proportion of available height (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn fill_top(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        self.fill(Side::Top, ratio, func);
    }

    /// Creates a frame on the right taking a proportion of available width.
    /// # Parameters
    /// * `ratio` - Proportion of available width (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn fill_right(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        self.fill(Side::Right, ratio, func);
    }

    /// Creates a frame on the bottom taking a proportion of available height.
    /// # Parameters
    /// * `ratio` - Proportion of available height (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn fill_bottom(&mut self, ratio: f32, func: impl FnMut(&mut Frame)) {
        self.fill(Side::Bottom, ratio, func);
    }
}

/// Shrinks a rectangle by applying a margin on all sides.
#[inline(always)]
fn rect_shrink(rect: Rect, margin: u16) -> Rect {
    Rect {
        x: rect.x + margin,
        y: rect.y + margin,
        w: rect.w.saturating_sub(margin * 2),
        h: rect.h.saturating_sub(margin * 2),
    }
}

/// Expands a rectangle by removing a margin from all sides.
#[inline(always)]
fn rect_expand(rect: Rect, margin: u16) -> Rect {
    Rect {
        x: rect.x - margin,
        y: rect.y - margin,
        w: rect.w.saturating_add(margin * 2),
        h: rect.h.saturating_add(margin * 2),
    }
}
