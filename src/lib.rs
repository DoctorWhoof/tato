#![no_std]

// TODO: Use in a real world project, adjust with any necessary improvements to support it.

// TODO: (?) Determine how much rects are overlapping if Layout is too small, and shrink each one accordingly
// OR return result with difference


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
    /// Controls whether the children rects are culled right before or right after the limit.
    /// Set to "false" if you're going to clip the rect's graphics only when you draw them, "true"
    /// if you don't want to clip anything (rect will simply disappear when touches the parent's edge )
    pub aggressive_culling: bool,
}

/// Represents a rectangle with position and dimensions.
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
            aggressive_culling: true,
        }
    }

    /// The rect that represents this Frame's position and size.
    /// Does not change when adding child frames.
    pub fn rect(&self) -> Rect<T> {
        self.rect
    }

    /// The available space to add more child frames.
    /// Shrinks every time a child frame is added.
    pub fn cursor(&self) -> Rect<T> {
        self.cursor
    }

    /// Sets a new margin value and recalculates the cursor rectangle.
    pub fn set_margin(&mut self, margin: T) {
        //remove old margin
        self.cursor = rect_expand(self.cursor, self.margin);
        // apply new margin
        self.margin = margin;
        self.cursor = rect_shrink(self.rect, self.margin);
    }

    /// Returns the current margin value.
    pub fn margin(&self) -> T {
        self.margin
    }

    /// Sets a new margin value and recalculates the cursor rectangle.
    pub fn set_gap(&mut self, gap: T) {
        self.gap = gap
    }

    /// Returns the current margin value.
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

    fn add_scope(&mut self, side: Side, len: T, scale: f32, mut func: impl FnMut(&mut Frame<T>)) {
        let scaled_len = T::from_f32(len.to_f32() * scale);
        let margin = T::from_f32(self.gap.to_f32() * self.scale);
        let gap = T::from_f32(self.gap.to_f32() * self.scale);

        if self.cursor.h < self.margin || self.cursor.w < self.margin {
            return;
        }

        // Calculate the child rectangle based on the side
        let child_rect = match side {
            Side::Left => {
                if self.cursor.x > self.rect.x + self.rect.w {
                    return;
                }
                Rect {
                    x: self.cursor.x,
                    y: self.cursor.y,
                    w: scaled_len,
                    h: self.cursor.h,
                }
            }
            Side::Right => Rect {
                x: (self.cursor.x + self.cursor.w).saturating_sub(scaled_len),
                y: self.cursor.y,
                w: scaled_len,
                h: self.cursor.h,
            },
            Side::Top => {
                if self.cursor.y > self.rect.y + self.rect.h {
                    return;
                }
                Rect {
                    x: self.cursor.x,
                    y: self.cursor.y,
                    w: self.cursor.w,
                    h: scaled_len,
                }
            }
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

        if self.aggressive_culling {
            let parent_dimension = if is_horizontal {
                self.cursor.w
            } else {
                self.cursor.h
            };
            if dimension > parent_dimension {
                return;
            }
        }

        // Update parent cursor
        match side {
            Side::Left => {
                self.cursor.x += scaled_len + gap;
                self.cursor.w = self.cursor.w.saturating_sub(scaled_len + gap);
            }
            Side::Right => {
                self.cursor.w = self.cursor.w.saturating_sub(scaled_len + gap);
            }
            Side::Top => {
                self.cursor.y += scaled_len + gap;
                self.cursor.h = self.cursor.h.saturating_sub(scaled_len + gap);
            }
            Side::Bottom => {
                self.cursor.h = self.cursor.h.saturating_sub(scaled_len + gap);
            }
        }

        // Call the function with the new frame
        func(&mut Frame {
            rect: child_rect,
            cursor: child_cursor,
            margin: self.margin,
            gap: self.gap,
            scale: self.scale,
            aggressive_culling: self.aggressive_culling,
        })
    }

    /// Adds a new frame on the left side with specified width.
    /// # Parameters
    /// * `len` - Width of the new frame (before scaling)
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn add(&mut self, side: Side, len: T, func: impl FnMut(&mut Frame<T>)) {
        self.add_scope(side, len, self.scale, func)
    }

    /// Creates a frame on the specified side taking a proportion of available space.
    /// # Parameters
    /// * `side` - Which side to add the child frame to
    /// * `ratio` - Proportion of available space (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    pub fn fill(&mut self, side: Side, ratio: f32, func: impl FnMut(&mut Frame<T>)) {
        let is_horizontal = matches!(side, Side::Left | Side::Right);
        let len = if is_horizontal {
            self.cursor.w.to_f32() * ratio.clamp(0.0, 1.0)
        } else {
            self.cursor.h.to_f32() * ratio.clamp(0.0, 1.0)
        };

        self.add_scope(side, T::from_f32(len), 1.0, func);
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
