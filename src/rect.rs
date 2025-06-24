use crate::num::Num;

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

impl<T: Num> Rect<T> {

    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h }
    }

    /// Convert this rectangle to a floating-point rectangle for precise calculations
    pub fn to_f32(self) -> Rect<f32> {
        Rect {
            x: self.x.to_f32(),
            y: self.y.to_f32(),
            w: self.w.to_f32(),
            h: self.h.to_f32(),
        }
    }

    /// Convert a floating-point rectangle to this rectangle's number type
    pub fn from_f32(rect: Rect<f32>) -> Self {
        Self {
            x: T::from_f32(rect.x),
            y: T::from_f32(rect.y),
            w: T::from_f32(rect.w),
            h: T::from_f32(rect.h),
        }
    }

    /// Checks if the rectangle contains the point (x, y)
    pub fn contains(&self, x: T, y: T) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }

    /// Checks if this rectangle overlaps with another rectangle
    pub fn overlaps(&self, other: &Rect<T>) -> bool {
        let self_right = self.x + self.w;
        let self_bottom = self.y + self.h;
        let other_right = other.x + other.w;
        let other_bottom = other.y + other.h;

        self.x < other_right && other.x < self_right &&
        self.y < other_bottom && other.y < self_bottom
    }

    pub fn shrink(self, margin: T) -> Self {
        Self {
            x: self.x + margin,
            y: self.y + margin,
            w: T::get_max(self.w - (margin * T::two()), T::zero()),
            h: T::get_max(self.h - (margin * T::two()), T::zero()),
        }
    }

    pub fn expand(self, margin: T) -> Self {
        Self {
            x: self.x - margin,
            y: self.y - margin,
            w: self.w + margin * T::two(),
            h: self.h + margin * T::two(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains() {
        // Test with integers
        let rect = Rect { x: 10, y: 20, w: 30, h: 40 };

        // Points inside the rectangle
        assert!(rect.contains(10, 20));  // Top-left corner (inclusive)
        assert!(rect.contains(39, 59));  // Bottom-right corner (exclusive)
        assert!(rect.contains(25, 40));  // Middle point

        // Points outside the rectangle
        assert!(!rect.contains(9, 20));   // Left of rect
        assert!(!rect.contains(10, 19));  // Above rect
        assert!(!rect.contains(40, 20));  // Right of rect (exclusive)
        assert!(!rect.contains(10, 60));  // Below rect (exclusive)

        // Test with floating point
        let rect_f32 = Rect { x: 10.5, y: 20.5, w: 30.0, h: 40.0 };

        assert!(rect_f32.contains(10.5, 20.5));      // Top-left (inclusive)
        assert!(rect_f32.contains(40.49, 60.49));    // Almost bottom-right
        assert!(!rect_f32.contains(40.5, 60.5));     // Exactly bottom-right (exclusive)
    }

    #[test]
    fn test_overlaps() {
        // Test with integers
        let rect1 = Rect { x: 10, y: 20, w: 30, h: 40 };

        // Completely overlapping (same rect)
        let rect2 = Rect { x: 10, y: 20, w: 30, h: 40 };
        assert!(rect1.overlaps(&rect2));

        // Partial overlap
        let rect3 = Rect { x: 5, y: 15, w: 10, h: 10 };
        assert!(rect1.overlaps(&rect3));

        // Contained within
        let rect4 = Rect { x: 15, y: 25, w: 10, h: 10 };
        assert!(rect1.overlaps(&rect4));

        // Adjacent but not overlapping (left)
        let rect5 = Rect { x: 0, y: 20, w: 10, h: 40 };
        assert!(!rect1.overlaps(&rect5));

        // Adjacent but not overlapping (top)
        let rect6 = Rect { x: 10, y: 0, w: 30, h: 20 };
        assert!(!rect1.overlaps(&rect6));

        // Completely separate
        let rect7 = Rect { x: 100, y: 100, w: 10, h: 10 };
        assert!(!rect1.overlaps(&rect7));

        // Test with floating point
        let rect_f1 = Rect { x: 10.5, y: 20.5, w: 30.0, h: 40.0 };
        let rect_f2 = Rect { x: 40.4, y: 20.5, w: 5.0, h: 5.0 };
        assert!(rect_f1.overlaps(&rect_f2));  // Barely overlapping

        let rect_f3 = Rect { x: 40.5, y: 20.5, w: 5.0, h: 5.0 };
        assert!(!rect_f1.overlaps(&rect_f3));  // Just touching but not overlapping
    }

    #[test]
    fn test_to_f32_and_from_f32() {
        let rect_u32 = Rect { x: 10u32, y: 20u32, w: 30u32, h: 40u32 };
        let rect_f32 = rect_u32.to_f32();

        assert_eq!(rect_f32.x, 10.0);
        assert_eq!(rect_f32.y, 20.0);
        assert_eq!(rect_f32.w, 30.0);
        assert_eq!(rect_f32.h, 40.0);

        let rect_i32 = Rect::<i32>::from_f32(rect_f32);
        assert_eq!(rect_i32.x, 10);
        assert_eq!(rect_i32.y, 20);
        assert_eq!(rect_i32.w, 30);
        assert_eq!(rect_i32.h, 40);
    }

    #[test]
    fn test_shrink_and_expand() {
        let rect = Rect { x: 10.0, y: 20.0, w: 30.0, h: 40.0 };

        // Test shrink
        let shrunk = rect.shrink(5.0);
        assert_eq!(shrunk.x, 15.0);
        assert_eq!(shrunk.y, 25.0);
        assert_eq!(shrunk.w, 20.0);
        assert_eq!(shrunk.h, 30.0);

        // Test shrink to zero size
        let shrunk_zero = rect.shrink(20.0);
        assert_eq!(shrunk_zero.x, 30.0);
        assert_eq!(shrunk_zero.y, 40.0);
        assert_eq!(shrunk_zero.w, 0.0);
        assert_eq!(shrunk_zero.h, 0.0);

        // Test expand
        let expanded = rect.expand(5.0);
        assert_eq!(expanded.x, 5.0);
        assert_eq!(expanded.y, 15.0);
        assert_eq!(expanded.w, 40.0);
        assert_eq!(expanded.h, 50.0);
    }
}
