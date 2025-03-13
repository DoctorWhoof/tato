#![no_std]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]

mod num;
pub use num::*;

/// A layout frame that manages rectangular areas with margins and scaling.
/// A frame consists of an outer rectangle, an inner cursor rectangle (available space),
/// and properties that control how child frames are created and positioned.
#[derive(Debug, Clone)]
pub struct Frame<T> {
    /// The outer rectangle defining the frame boundaries
    rect: Rect<T>,
    /// Inner rectangle representing available space
    cursor: Rect<T>,
    /// Scaling factor for dimensions
    scale: f32,
    /// Margin size between frames
    margin: T,
    /// Gap between each child frame
    gap: T,
    /// Controls how children rects are culled when they exceed available space
    pub culling: Culling,
}

/// Represents a generic rectangle with position and dimensions that
/// implement [Num] trait, i.e. u16, f32, etc.
///
/// A rectangle is defined by its top-left corner coordinates (x, y)
/// and its width and height.
#[derive(Debug, Clone, Copy)]
pub struct Rect<T> {
    /// X-coordinate of the top-left corner
    pub x: T,
    /// Y-coordinate of the top-left corner
    pub y: T,
    /// Width of the rectangle
    pub w: T,
    /// Height of the rectangle
    pub h: T,
}

/// Represents the side of a frame where a child frame can be added.
#[derive(Debug, Clone, Copy)]
pub enum Side {
    /// Left side of the frame
    Left,
    /// Right side of the frame
    Right,
    /// Top side of the frame
    Top,
    /// Bottom side of the frame
    Bottom,
}

/// Clipping strategy
#[derive(Debug, Clone, Copy)]
pub enum Culling {
    /// Allows child frame even if it goes over the available space.
    Relaxed,
    /// Removes child frames that touch the margin.
    Aggressive,
    /// Clamps child frame's edges to available space.
    Clamp,
}

impl<T> Frame<T>
where
    T: Num,
{
    /// Creates a new frame with the specified outer rectangle.
    /// Initializes with default values for scale (1.0) and margin (5 pixels).
    pub fn new(rect: Rect<T>) -> Self {
        let scale = 1.0;
        let margin = T::four();
        let cursor = rect_shrink(rect, margin);
        Self {
            rect,
            cursor,
            margin,
            gap: margin,
            scale,
            culling: Culling::Aggressive,
        }
    }

    /// The rect that represents this Frame's position and size.
    /// Does not change when adding child frames.
    pub fn rect(&self) -> Rect<T> {
        self.rect
    }

    /// The available space to push more child frames.
    /// Shrinks every time a child frame is added.
    pub fn cursor(&self) -> Rect<T> {
        self.cursor
    }

    /// Sets a new margin value and recalculates the cursor rectangle.
    pub fn set_margin(&mut self, margin: T) {
        // Remove old margin
        self.cursor = rect_expand(self.cursor, self.margin);
        // Apply new margin
        self.margin = margin;
        self.cursor = rect_shrink(self.rect, self.margin);
    }

    /// Returns the current margin value.
    pub fn margin(&self) -> T {
        self.margin
    }

    /// Sets a new gap value.
    pub fn set_gap(&mut self, gap: T) {
        self.gap = gap
    }

    /// Returns the current gap value.
    pub fn gap(&self) -> T {
        self.gap
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

    /// Adds a new frame on the specified side with specified length.
    /// # Parameters
    /// * `side` - Which side to add the child frame to
    /// * `len` - Length of the new frame
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn push(&mut self, side: Side, len: T, func: impl FnMut(&mut Frame<T>)) {
        let is_horizontal = matches!(side, Side::Left | Side::Right);

        // Default width and height based on the side
        let (w, h) = if is_horizontal {
            (len, self.cursor.h / T::from_f32(self.scale))
        } else {
            (self.cursor.w / T::from_f32(self.scale), len)
        };

        // Default offset is zero
        let offset_x = T::zero();
        let offset_y = T::zero();

        self.add_scope(side, w, h, offset_x, offset_y, self.scale, true, func);
    }

    /// Creates a frame on the specified side taking a proportion of available space.
    /// # Parameters
    /// * `side` - Which side to add the child frame to
    /// * `ratio` - Proportion of available space (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    pub fn fill(&mut self, side: Side, ratio: f32, func: impl FnMut(&mut Frame<T>)) {
        let is_horizontal = matches!(side, Side::Left | Side::Right);

        let (w, h) = if is_horizontal {
            let len = self.cursor.w.to_f32() * ratio.clamp(0.0, 1.0);
            (T::from_f32(len), self.cursor.h)
        } else {
            let len = self.cursor.h.to_f32() * ratio.clamp(0.0, 1.0);
            (self.cursor.w, T::from_f32(len))
        };

        // Default offset is zero
        let offset_x = T::zero();
        let offset_y = T::zero();

        self.add_scope(side, w, h, offset_x, offset_y, 1.0, true, func);
    }

    /// Creates a centered frame with specific dimensions. Does not modify the cursor!
    /// # Parameters
    /// * `w` - Width of the new frame
    /// * `h` - Height of the new frame
    /// * `func` - Closure to execute with the new child frame
    pub fn center(&mut self, w: T, h: T, func: impl FnMut(&mut Frame<T>)) {
        // Calculate the centering offsets
        let offset_x = (self.cursor.w - w) / T::two();
        let offset_y = (self.cursor.h - h) / T::two();

        // Ensure offsets are non-negative
        let x = if offset_x < T::zero() {
            T::zero()
        } else {
            offset_x
        };
        let y = if offset_y < T::zero() {
            T::zero()
        } else {
            offset_y
        };

        self.add_scope(Side::Left, w, h, x, y, self.scale, false, func);
    }

    /// Creates a centered frame that takes a proportion of the available space.
    /// Does not modify the available space!
    /// # Parameters
    /// * `x_ratio` - Proportion of available width (0.0 to 1.0)
    /// * `y_ratio` - Proportion of available height (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    pub fn center_fill(&mut self, x_ratio: T, y_ratio: T, func: impl FnMut(&mut Frame<T>)) {
        // Calculate width and height based on ratios
        let w = self.cursor.w.to_f32() * x_ratio.to_f32().clamp(0.0, 1.0);
        let h = self.cursor.h.to_f32() * y_ratio.to_f32().clamp(0.0, 1.0);

        // Calculate the centering offsets - should be half of the remaining space, not half the width
        let offset_x = (self.cursor.w.to_f32() - w) / 2.0;
        let offset_y = (self.cursor.h.to_f32() - h) / 2.0;

        // Ensure offsets are non-negative
        let x = if offset_x < 0.0 {
            T::zero()
        } else {
            T::from_f32(offset_x)
        };
        let y = if offset_y < 0.0 {
            T::zero()
        } else {
            T::from_f32(offset_y)
        };

        self.add_scope(Side::Left, T::from_f32(w), T::from_f32(h), x, y, self.scale, false, func);
    }

    /// Allows arbitrary placement of the new frame in relation to the current frame.
    /// Does not modify the available space!
    /// # Parameters
    /// * `x` - X position of the new frame in relation to this frame.
    /// * `y` - Y position of the new frame in relation to this frame.
    /// * `w` - Width of the new frame
    /// * `h` - Height of the new frame
    /// * `func` - Closure to execute with the new child frame
    pub fn place(&mut self, side: Side, x: T, y: T, w: T, h: T, func: impl FnMut(&mut Frame<T>)) {
        // Ensures "self.scale" is used. "fill" always adds with scale = 1.0 instead.
        self.add_scope(side, w, h, x, y, self.scale, false, func);
    }

    /// Internal jack-of-all-trades function called by the mode specialized public functions
    fn add_scope(
        &mut self,
        side: Side,
        w: T,
        h: T,
        extra_x: T,
        extra_y: T,
        scale: f32,
        update_cursor: bool,
        mut func: impl FnMut(&mut Frame<T>),
    ) {
        let scaled_w = T::from_f32(w.to_f32() * scale);
        let scaled_h = T::from_f32(h.to_f32() * scale);
        let margin = T::from_f32(self.gap.to_f32() * self.scale);
        let gap = T::from_f32(self.gap.to_f32() * self.scale);

        // Calculate the child rectangle based on the side
        let mut child_rect = match side {
            Side::Left => {
                if self.cursor.x > self.rect.x + self.rect.w {
                    return;
                }
                Rect {
                    x: self.cursor.x + extra_x,
                    y: self.cursor.y + extra_y,
                    w: scaled_w,
                    h: scaled_h,
                }
            }
            Side::Right => Rect {
                x: (self.cursor.x + self.cursor.w).saturating_sub(scaled_w) - extra_x,
                y: self.cursor.y + extra_y,
                w: scaled_w,
                h: scaled_h,
            },
            Side::Top => {
                if self.cursor.y > self.rect.y + self.rect.h {
                    return;
                }
                Rect {
                    x: self.cursor.x + extra_x,
                    y: self.cursor.y + extra_y,
                    w: scaled_w,
                    h: scaled_h,
                }
            }
            Side::Bottom => Rect {
                x: self.cursor.x + extra_x,
                y: (self.cursor.y + self.cursor.h).saturating_sub(scaled_h) - extra_y,
                w: scaled_w,
                h: scaled_h,
            },
        };

        match self.culling {
            Culling::Relaxed => {}
            Culling::Aggressive => {
                if child_rect.w.round_down() > self.cursor.w.round_up() {
                    return;
                }
                if child_rect.h.round_down() > self.cursor.h.round_up() {
                    return;
                }
            }
            Culling::Clamp => {
                // Clamp to ensure the rect stays within cursor boundaries
                // Clamp x position
                if child_rect.x < self.cursor.x {
                    let diff = self.cursor.x - child_rect.x;
                    child_rect.x = self.cursor.x;
                    child_rect.w = child_rect.w.saturating_sub(diff);
                }

                // Clamp y position
                if child_rect.y < self.cursor.y {
                    let diff = self.cursor.y - child_rect.y;
                    child_rect.y = self.cursor.y;
                    child_rect.h = child_rect.h.saturating_sub(diff);
                }

                // Clamp width
                if child_rect.x + child_rect.w > self.cursor.x + self.cursor.w {
                    child_rect.w = self.cursor.x + self.cursor.w - child_rect.x;
                }

                // Clamp height
                if child_rect.y + child_rect.h > self.cursor.y + self.cursor.h {
                    child_rect.h = self.cursor.y + self.cursor.h - child_rect.y;
                }
            }
        }

        if child_rect.w < T::one() || child_rect.h < T::one() {
            return;
        }

        // Update parent cursor
        if update_cursor {
            match side {
                Side::Left => {
                    // Add extra_x to the cursor movement
                    self.cursor.x += scaled_w + gap + extra_x;
                    self.cursor.w = self.cursor.w.saturating_sub(scaled_w + gap + extra_x);
                }
                Side::Right => {
                    // Subtract extra_x in width reduction
                    self.cursor.w = self.cursor.w.saturating_sub(scaled_w + gap + extra_x);
                }
                Side::Top => {
                    // Add extra_y to the cursor movement
                    self.cursor.y += scaled_h + gap + extra_y;
                    self.cursor.h = self.cursor.h.saturating_sub(scaled_h + gap + extra_y);
                }
                Side::Bottom => {
                    // Subtract extra_y in height reduction
                    self.cursor.h = self.cursor.h.saturating_sub(scaled_h + gap + extra_y);
                }
            }
        }

        // Call the function with the new frame
        func(&mut Frame {
            cursor: rect_shrink(child_rect, margin),
            rect: child_rect,
            margin: self.margin,
            gap: self.gap,
            scale: self.scale,
            culling: self.culling,
        })
    }
}

/// Shrinks a rectangle by applying a margin on all sides.
#[inline(always)]
fn rect_shrink<T>(rect: Rect<T>, margin: T) -> Rect<T>
where
    T: Num,
{
    Rect {
        x: rect.x + margin,
        y: rect.y + margin,
        w: rect.w.saturating_sub(margin * T::two()),
        h: rect.h.saturating_sub(margin * T::two()),
    }
}

/// Expands a rectangle by removing a margin from all sides.
#[inline(always)]
fn rect_expand<T>(rect: Rect<T>, margin: T) -> Rect<T>
where
    T: Num,
{
    Rect {
        x: rect.x - margin,
        y: rect.y - margin,
        w: rect.w.saturating_add(margin * T::two()),
        h: rect.h.saturating_add(margin * T::two()),
    }
}
