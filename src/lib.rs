#![no_std]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/readme.md"))]

mod num;
pub use num::*;

/// shortens signature for a mutable frame reference
macro_rules! child {
    () => {
        impl FnMut(&mut Frame<T>)
    };
}

pub trait Child<T>: FnMut(&mut Frame<T>) {}

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
    pub fitting: Fitting,
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
#[derive(Debug, Clone, Copy, Default)]
pub enum Edge {
    #[default]
    /// Left side of the frame
    Left,
    /// Right side of the frame
    Right,
    /// Top side of the frame
    Top,
    /// Bottom side of the frame
    Bottom,
}

/// Represents the alignment of a child frame that is sized (width, height).
/// Notice that LeftTop is *not* the same as TopLeft! LeftTop means "push_edge from the left,
/// align to Top" and TopLeft means "push_edge from Top, align to Left". The result may look the same,
/// but the availble space will shrink from the left in the former, from the top in the latter.
#[derive(Debug, Clone, Copy, Default)]
pub enum Align {
    #[default]
    LeftTop,
    LeftCenter,
    LeftBottom,
    RightTop,
    RightCenter,
    RightBottom,
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    /// Only option that does not shrink the available space (the "cursor" rect).
    Center,
}

/// Clipping strategy
#[derive(Debug, Clone, Copy)]
pub enum Fitting {
    /// Allows child frame even if it goes over the available space.
    /// Also useful for debugging, since Frame is less likely to disappear when space is too small.
    Relaxed,
    /// Removes child frames that touch the margin.
    Aggressive,
    /// Clamps child frame's edges to available space.
    Clamp,
    /// Scales the child frame to fit available space while preserving aspect ratio.
    Scale,
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
            fitting: Fitting::Aggressive,
        }
    }

    /// The rect that represents this Frame's position and size.
    /// Does not change when adding child frames.
    pub fn rect(&self) -> Rect<T> {
        self.rect
    }

    /// The available space to push_edge more child frames.
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

    /// Determine the edge associated with an alignment
    fn alignment_to_edge(align: Align) -> Edge {
        match align {
            Align::LeftTop | Align::LeftCenter | Align::LeftBottom => Edge::Left,
            Align::RightTop | Align::RightCenter | Align::RightBottom => Edge::Right,
            Align::TopLeft | Align::TopCenter | Align::TopRight => Edge::Top,
            Align::BottomLeft | Align::BottomCenter | Align::BottomRight => Edge::Bottom,
            Align::Center => Edge::Left, // Center uses left as base for positioning
        }
    }

    fn edge_to_alignment(edge: Edge) -> Align {
        match edge {
            Edge::Left => Align::LeftTop,
            Edge::Right => Align::RightTop,
            Edge::Top => Align::TopLeft,
            Edge::Bottom => Align::BottomLeft,
        }
    }

    /// Calculate offsets based on alignment for positioned frames
    fn calculate_align_offsets(&self, align: Align, w: T, h: T) -> (T, T, bool) {
        let update_cursor = !matches!(align, Align::Center);
        let (offset_x, offset_y) = match align {
            // Left side alignments
            Align::LeftTop => (T::zero(), T::zero()),
            Align::LeftCenter => (T::zero(), (self.cursor.h - h) / T::two()),
            Align::LeftBottom => (T::zero(), self.cursor.h.saturating_sub(h)),

            // Right side alignments
            Align::RightTop => (T::zero(), T::zero()),
            Align::RightCenter => (T::zero(), (self.cursor.h - h) / T::two()),
            Align::RightBottom => (T::zero(), self.cursor.h.saturating_sub(h)),

            // Top side alignments
            Align::TopLeft => (T::zero(), T::zero()),
            Align::TopCenter => ((self.cursor.w - w) / T::two(), T::zero()),
            Align::TopRight => (self.cursor.w.saturating_sub(w), T::zero()),

            // Bottom side alignments
            Align::BottomLeft => (T::zero(), T::zero()),
            Align::BottomCenter => ((self.cursor.w - w) / T::two(), T::zero()),
            Align::BottomRight => (self.cursor.w.saturating_sub(w), T::zero()),

            // Center alignment
            Align::Center => (
                (self.cursor.w - w) / T::two(),
                (self.cursor.h - h) / T::two(),
            ),
        };

        // Ensure offsets are non-negative
        let x = offset_x.get_max(T::zero());
        let y = offset_y.get_max(T::zero());

        (x, y, update_cursor)
    }

    /// Calculate the scale needed to fit a rectangle of given dimensions
    /// into the available space, preserving aspect ratio.
    /// Takes into account the offsets where the rectangle will be placed.
    fn calculate_fit_scale(&self, w: T, h: T, offset_x: T, offset_y: T) -> f32 {
        let original_w = w.to_f32();
        let original_h = h.to_f32();

        // Calculate available space considering offsets
        let available_w = self.cursor.w.to_f32() - offset_x.to_f32();
        let available_h = self.cursor.h.to_f32() - offset_y.to_f32();

        // Calculate actual scale to use, ensuring it won't exceed available space
        // First determine what scale we need to fit the content
        let fit_w_scale = if original_w > 0.0 {
            available_w / (original_w * self.scale)
        } else {
            1.0
        };
        let fit_h_scale = if original_h > 0.0 {
            available_h / (original_h * self.scale)
        } else {
            1.0
        };
        let fit_scale = fit_w_scale.min(fit_h_scale).clamp(0.0, 1.0);

        // If self.scale is > 1.0, we need to ensure we still scale down if needed
        // if self.scale > 1.0 {
        //     // Use the smaller of: requested scale or the maximum scale that fits
        //     self.scale.min(fit_scale * self.scale)
        // } else {
        // For self.scale <= 1.0, use the smaller of: requested scale or fit_scale
        // self.scale * fit_scale
        // }
        match self.fitting {
            Fitting::Relaxed | Fitting::Aggressive | Fitting::Clamp => 1.0,
            Fitting::Scale => self.scale * fit_scale,
        }
    }

    /// Attempts to push_edge a rect with size (w,h), if there isn't enough available space, the rect
    /// is scaled down preserving the aspect ratio.
    /// # Parameters
    /// * `align` - Alignment that determines positioning and cursor updating
    /// * `w` - Width of the new frame
    /// * `h` - Height of the new frame
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn push_size(&mut self, align: Align, w: T, h: T, func: child!()) {
        let (offset_x, offset_y, update_cursor) = self.calculate_align_offsets(align, w, h);
        let side = Self::alignment_to_edge(align);

        // Calculate actual scale and apply it to dimensions, taking offsets into account
        let actual_scale = self.calculate_fit_scale(w, h, offset_x, offset_y);
        let scaled_w = T::from_f32(w.to_f32() * actual_scale);
        let scaled_h = T::from_f32(h.to_f32() * actual_scale);

        self.add_scope(
            side,
            offset_x,
            offset_y,
            scaled_w,
            scaled_h,
            1.0,
            update_cursor,
            self.fitting,
            func,
        );
    }

    /// Adds a new frame on the specified side with specified length.
    /// # Parameters
    /// * `side` - Which side to add the child frame to
    /// * `len` - Length of the new frame
    /// * `func` - Closure to execute with the new child frame
    #[inline(always)]
    pub fn push_edge(&mut self, edge: Edge, len: T, func: child!()) {
        // Default width and height based on the side
        let is_horizontal = matches!(edge, Edge::Left | Edge::Right);
        let (w, h) = if is_horizontal {
            (len, self.cursor.h / T::from_f32(self.scale))
        } else {
            (self.cursor.w / T::from_f32(self.scale), len)
        };

        let align = Self::edge_to_alignment(edge);
        self.push_size(align, w, h, func);
    }

    /// Creates a frame on the specified side taking a proportion of the original available space,
    /// not the current available space. This is more intuitive, i.e. if you want to divide a Frame
    /// into 4 smaller frames just fill it four times using ratio = 0.25.
    /// # Parameters
    /// * `align` - Alignment that determines positioning and cursor updating
    /// * `ratio_x` - Proportion of original available width (0.0 to 1.0)
    /// * `ratio_y` - Proportion of original available height (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    pub fn fill_size(&mut self, align: Align, ratio_x: f32, ratio_y: f32, func: child!()) {
        let side = Self::alignment_to_edge(align);
        let update_cursor = !matches!(align, Align::Center);

        // Calculate available width and height after respecting margins
        let max_w = self.cursor.w.to_f32();
        let max_h = self.cursor.h.to_f32();

        let original_cursor = rect_shrink(self.rect, self.margin);

        let w = T::from_f32((original_cursor.w.to_f32() * ratio_x.clamp(0.0, 1.0)).clamp(0.0, max_w));
        let h = T::from_f32((original_cursor.h.to_f32() * ratio_y.clamp(0.0, 1.0)).clamp(0.0, max_h));

        let (offset_x, offset_y, _) = self.calculate_align_offsets(align, w, h);

        self.add_scope(
            side,
            offset_x,
            offset_y,
            w,
            h,
            1.0,
            update_cursor,
            self.fitting,
            func,
        );
    }

    /// Creates a frame on the specified side taking a proportion of the original available space,
    /// not the current available space. This is more intuitive, i.e. if you want to divide a Frame
    /// into 4 smaller frames just fill it four times using ratio = 0.25.
    /// # Parameters
    /// * `align` - Alignment that determines positioning and cursor updating
    /// * `ratio_x` - Proportion of original available width (0.0 to 1.0)
    /// * `ratio_y` - Proportion of original available height (0.0 to 1.0)
    /// * `func` - Closure to execute with the new child frame
    pub fn fill_edge(&mut self, edge: Edge, ratio: f32, func: child!()) {
        let align = Self::edge_to_alignment(edge);
        // Calculate available width and height after respecting margins
        let is_horizontal = matches!(edge, Edge::Left | Edge::Right);
        let (ratio_x, ratio_y) = if is_horizontal {
            (ratio, 1.0)
        } else {
            (1.0, ratio)
        };
        self.fill_size(align, ratio_x, ratio_y, func);
    }

    /// Allows arbitrary placement of the new frame in relation to the current frame.
    /// Does not modify the available space by default, unless Align is not Center.
    /// Scales the frame if necessary to fit.
    /// # Parameters
    /// * `align` - Alignment that determines cursor updating
    /// * `x` - X position of the new frame in relation to this frame.
    /// * `y` - Y position of the new frame in relation to this frame.
    /// * `w` - Width of the new frame
    /// * `h` - Height of the new frame
    /// * `func` - Closure to execute with the new child frame
    pub fn place(&mut self, align: Align, x: T, y: T, w: T, h: T, func: child!()) {
        let side = Self::alignment_to_edge(align);
        let update_cursor = !matches!(align, Align::Center);

        // Calculate actual scale and apply it to dimensions, taking offsets into account
        let actual_scale = self.calculate_fit_scale(w, h, x, y);
        let scaled_w = T::from_f32(w.to_f32() * actual_scale);
        let scaled_h = T::from_f32(h.to_f32() * actual_scale);

        // Ensures "1.0" is used as scale since we've already applied scaling to dimensions
        self.add_scope(
            side,
            x,
            y,
            scaled_w,
            scaled_h,
            1.0,
            update_cursor,
            self.fitting,
            func,
        );
    }

    /// Internal jack-of-all-trades function called by the mode specialized public functions
    fn add_scope(
        &mut self,
        side: Edge,
        extra_x: T,
        extra_y: T,
        w: T,
        h: T,
        scale: f32,
        update_cursor: bool,
        fitting: Fitting,
        mut func: child!(),
    ) {
        let scaled_w = T::from_f32(w.to_f32() * scale);
        let scaled_h = T::from_f32(h.to_f32() * scale);
        let margin = T::from_f32(self.gap.to_f32() * self.scale);
        let gap = T::from_f32(self.gap.to_f32() * self.scale);

        // Calculate the child rectangle based on the side
        let mut child_rect = match side {
            Edge::Left => {
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
            Edge::Right => Rect {
                x: (self.cursor.x + self.cursor.w).saturating_sub(scaled_w) - extra_x,
                y: self.cursor.y + extra_y,
                w: scaled_w,
                h: scaled_h,
            },
            Edge::Top => {
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
            Edge::Bottom => Rect {
                x: self.cursor.x + extra_x,
                y: (self.cursor.y + self.cursor.h).saturating_sub(scaled_h) - extra_y,
                w: scaled_w,
                h: scaled_h,
            },
        };

        match self.fitting {
            Fitting::Relaxed => {}
            Fitting::Aggressive => {
                if (child_rect.x + child_rect.w).round_down()
                    > (self.cursor.x + self.cursor.w).round_up()
                {
                    return;
                }
                if (child_rect.y + child_rect.h).round_down()
                    > (self.cursor.y + self.cursor.h).round_up()
                {
                    return;
                }
            }
            Fitting::Clamp | Fitting::Scale => {
                // For Clamp, we adjust the rectangle
                // For Scale, we've already handled scaling in the caller methods
                if matches!(self.fitting, Fitting::Clamp) {
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
        }

        if child_rect.w < T::one() || child_rect.h < T::one() {
            return;
        }

        // Update parent cursor
        if update_cursor {
            match side {
                Edge::Left => {
                    // Add extra_x to the cursor movement
                    self.cursor.x += scaled_w + gap + extra_x;
                    self.cursor.w = self.cursor.w.saturating_sub(scaled_w + gap + extra_x);
                }
                Edge::Right => {
                    // Subtract extra_x in width reduction
                    self.cursor.w = self.cursor.w.saturating_sub(scaled_w + gap + extra_x);
                }
                Edge::Top => {
                    // Add extra_y to the cursor movement
                    self.cursor.y += scaled_h + gap + extra_y;
                    self.cursor.h = self.cursor.h.saturating_sub(scaled_h + gap + extra_y);
                }
                Edge::Bottom => {
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
            fitting,
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
