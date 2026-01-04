#![warn(clippy::std_instead_of_core, clippy::std_instead_of_alloc)]
#![no_std]

use tato_math::Num;

/// Fast fixed size ring buffer with O(1) basic statistical functions
#[derive(Debug, Clone)]
pub struct AvgBuffer<const CAP: usize, T: Num> {
    data: [T; CAP],
    head: usize,
    sum: Option<T>,
    max: Option<T>,
    min: Option<T>,
    filled_len: usize,
    dirty_minmax: bool,
}

impl<const CAP: usize, T: Num> Default for AvgBuffer<CAP, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const CAP: usize, T: Num> AvgBuffer<CAP, T> {
    /// Creates a new, empty buffer.
    pub fn new() -> Self {
        assert!(CAP > 0, "AvgBuffer Error: Capacity must be larger than zero");
        AvgBuffer {
            data: [T::default(); CAP],
            head: 0,
            sum: None,
            max: None,
            min: None,
            filled_len: 0,
            dirty_minmax: false,
        }
    }

    /// Creates a new buffer pre-populated with a value, filled to capacity.
    pub fn pre_filled(value: T) -> Self {
        assert!(CAP > 0, "AvgBuffer Error: Capacity must be larger than zero");
        AvgBuffer {
            data: [value; CAP],
            head: CAP - 1,
            sum: Some(value * T::from_usize_checked(CAP).unwrap()),
            max: Some(value),
            min: Some(value),
            filled_len: CAP,
            dirty_minmax: false,
        }
    }

    /// Fast! Sum is always kept up to date on push. No need to iterate.
    pub fn average(&self) -> T {
        if self.filled_len > 0 {
            return self.sum.unwrap_or(T::zero()) / T::from_usize_checked(self.filled_len).unwrap();
        }
        T::zero()
    }

    /// Resets buffer to its default empty state.
    pub fn clear(&mut self) {
        for n in 0..self.data.len() {
            self.data[n] = T::zero();
        }
        self.sum = None;
        self.max = None;
        self.min = None;
        self.filled_len = 0;
        self.head = 0;
    }

    /// True if buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.filled_len == 0
    }

    /// The largest value so far, if any.
    pub fn max(&mut self) -> T {
        if self.dirty_minmax {
            // Recalculate max only when needed
            self.recalculate_minmax();
        }
        self.max.unwrap_or(T::zero())
    }

    /// The smallest value so far, if any.
    pub fn min(&mut self) -> T {
        if self.dirty_minmax {
            // Recalculate min only when needed
            self.recalculate_minmax();
        }
        self.min.unwrap_or(T::zero())
    }

    fn recalculate_minmax(&mut self) {
        if self.filled_len == 0 {
            self.min = None;
            self.max = None;
            return;
        }

        // First find the index of the oldest element
        let start_idx =
            if self.filled_len < CAP { 0 } else { (self.head + CAP - self.filled_len) % CAP };

        // Initialize with the first valid element
        let mut new_min = self.data[start_idx];
        let mut new_max = self.data[start_idx];

        // Check all valid elements
        for i in 1..self.filled_len {
            let idx = (start_idx + i) % CAP;
            new_min = T::get_min(new_min, self.data[idx]);
            new_max = T::get_max(new_max, self.data[idx]);
        }

        self.min = Some(new_min);
        self.max = Some(new_max);
        self.dirty_minmax = false;
    }

    /// The maximum number of items. Older items are discarded in favor of newer ones
    /// if capacity is exceeded.
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Current value count, will always be lower or equal to capacity.
    pub fn len(&self) -> usize {
        self.filled_len
    }

    /// Push a single value.
    pub fn push(&mut self, value: T) {

        // Fast path for sum calculation
        if self.filled_len < CAP {
            self.sum = match self.sum {
                None => Some(value),
                Some(sum) => Some(sum + value),
            };
        } else {
            // Buffer is full, subtract old value and add new
            let old_value = self.data[self.head];
            self.sum = Some(self.sum.unwrap_or(T::zero()) - old_value + value);
        }

        // Push data into storage
        self.data[self.head] = value;
        self.head = (self.head + 1) % CAP; // More efficient than if check
        if self.filled_len < CAP {
            self.filled_len += 1;
        }

        // Update min/max lazily
        if self.filled_len == CAP {
            // Only mark dirty if we're overwriting, not when adding
            self.dirty_minmax = true;
        } else {
            match self.max {
                None => self.max = Some(value),
                Some(max) if value > max => self.max = Some(value),
                _ => {},
            }

            match self.min {
                None => self.min = Some(value),
                Some(min) if value < min => self.min = Some(value),
                _ => {},
            }
        }
    }

    /// Pushes multiple values at once.
    pub fn push_slice(&mut self, slice: &[T]) {
        for item in slice {
            self.push(*item);
        }
    }

    /// Iterates through all values. Order of retrieval will likely NOT match order of input.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let head = self.head;
        let len = self.filled_len;
        let cap = CAP;

        (0..len).map(move |i| &self.data[(head + cap - len + i) % cap])
    }

    /// Range (max - min)
    pub fn range(&mut self) -> T {
        if self.filled_len == 0 {
            return T::zero();
        }
        self.max() - self.min()
    }

    /// Sum of all values
    pub fn sum(&self) -> T {
        self.sum.unwrap_or(T::zero())
    }

    /// Approximate median: (min + max) / 2
    pub fn approximate_median(&mut self) -> T {
        if self.filled_len == 0 {
            return T::zero();
        }
        if self.filled_len == 1 {
            return self.average();
        }

        // Simple approximation: (min + max) / 2
        let two = T::one() + T::one();
        (self.min() + self.max()) / two
    }

    /// True if second half of buffer has higher average than first half (requires iteration)
    pub fn is_trending_up(&self) -> bool {
        if self.filled_len < 4 {
            return false;
        }

        let half = self.filled_len / 2;
        let start_idx = if self.filled_len < CAP { 0 } else { (self.head + CAP - self.filled_len) % CAP };

        // Calculate first half sum
        let mut first_half_sum = T::zero();
        for i in 0..half {
            let idx = (start_idx + i) % CAP;
            first_half_sum = first_half_sum + self.data[idx];
        }

        // Calculate second half sum
        let mut second_half_sum = T::zero();
        for i in half..self.filled_len {
            let idx = (start_idx + i) % CAP;
            second_half_sum = second_half_sum + self.data[idx];
        }

        let remaining = self.filled_len - half;
        let first_avg = first_half_sum / T::from_usize_checked(half).unwrap();
        let second_avg = second_half_sum / T::from_usize_checked(remaining).unwrap();

        second_avg > first_avg
    }

    /// Mean absolute deviation (requires iteration)
    pub fn mean_absolute_deviation(&self) -> T {
        if self.filled_len == 0 {
            return T::zero();
        }

        let mean = self.average();
        let start_idx = if self.filled_len < CAP { 0 } else { (self.head + CAP - self.filled_len) % CAP };

        let mut sum_deviations = T::zero();
        for i in 0..self.filled_len {
            let idx = (start_idx + i) % CAP;
            let deviation = if self.data[idx] > mean {
                self.data[idx] - mean
            } else {
                mean - self.data[idx]
            };
            sum_deviations = sum_deviations + deviation;
        }

        sum_deviations / T::from_usize_checked(self.filled_len).unwrap()
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    const MARGIN: f32 = 0.000001;

    #[test]
    fn create_and_push() {
        const CAP: usize = 10;
        let mut buf = AvgBuffer::<CAP, f32>::new();
        for _ in 0..5 {
            buf.push(10.0);
        }

        assert_eq!(buf.capacity(), CAP);
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.average(), 10.0);

        for _ in 0..10 {
            buf.push(5.0);
        }
        assert_eq!(buf.len(), CAP);
        assert_eq!(buf.average(), 5.0);
    }

    #[test]
    fn clearing() {
        let mut buf = AvgBuffer::<10, f32>::new();
        for n in 0..buf.capacity() {
            buf.push(n as f32);
        }
        buf.clear();
        assert_eq!(buf.capacity(), 10);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.average(), 0.0);
        assert_eq!(buf.iter().next(), None);
    }

    #[test]
    fn iteration() {
        let mut buf = AvgBuffer::<10, f32>::new();
        let len = 7;
        for n in 0..len {
            buf.push(n as f32);
        }

        for (i, value) in buf.iter().enumerate() {
            assert_eq!(i as f32, *value);
        }
    }

    #[test]
    fn test_min_max_recalculation() {
        let mut buf = AvgBuffer::<5, f32>::new();

        // Fill buffer with increasing values
        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);
        buf.push(4.0);
        buf.push(5.0);

        // Initial min/max should be correct
        assert_eq!(buf.min(), 1.0);
        assert_eq!(buf.max(), 5.0);

        // Now overwrite the min value
        buf.push(2.5); // This overwrites 1.0

        // Min should be recalculated
        assert_eq!(buf.min(), 2.0);
        assert_eq!(buf.max(), 5.0);

        // Now overwrite the max value
        buf.push(3.5); // This overwrites 2.0
        buf.push(4.5); // This overwrites 3.0
        buf.push(3.0); // This overwrites 4.0
        buf.push(2.0); // This overwrites 5.0 (the max)

        // Max should be recalculated
        assert_eq!(buf.min(), 2.0);
        assert_eq!(buf.max(), 4.5);
    }

    #[test]
    fn test_buffer_wrapping() {
        let mut buf = AvgBuffer::<3, i32>::new();

        // Fill buffer
        buf.push(1);
        buf.push(2);
        buf.push(3);

        // Check initial state
        assert_eq!(buf.average(), 2);

        // Push more values to wrap around
        buf.push(4); // Overwrites 1
        assert_eq!(buf.average(), 3);

        buf.push(5); // Overwrites 2
        assert_eq!(buf.average(), 4);

        buf.push(6); // Overwrites 3
        assert_eq!(buf.average(), 5);

        // Check iteration order (should be in insertion order: 4,5,6)
        let mut collected = [0; 3];
        let mut count = 0;

        for &val in buf.iter() {
            if count < 3 {
                collected[count] = val;
                count += 1;
            }
        }

        assert_eq!(count, 3);
        assert_eq!(collected, [4, 5, 6]);
    }

    #[test]
    fn test_dirty_flag_behavior() {
        let mut buf = AvgBuffer::<5, f32>::new();

        // Fill buffer
        for i in 0..5 {
            buf.push(i as f32);
        }

        // First access should use cached values
        assert_eq!(buf.min(), 0.0);
        assert_eq!(buf.max(), 4.0);

        // Overwrite min
        buf.push(2.0); // Overwrites 0.0

        // Min should be recalculated
        assert_eq!(buf.min(), 1.0);

        // Overwrite multiple values including max
        buf.push(3.0); // Overwrites 1.0
        buf.push(2.0); // Overwrites 2.0
        buf.push(1.0); // Overwrites 3.0
        buf.push(0.5); // Overwrites 4.0 (max)

        // Max should be recalculated
        assert_eq!(buf.max(), 3.0);
    }

    #[test]
    fn test_pre_filled_edge_cases() {
        // Test with very large values
        let buf_large = AvgBuffer::<5, f32>::pre_filled(1e10);
        assert!((buf_large.average() - 1e10).abs() < MARGIN);

        // Test with very small values
        let buf_small = AvgBuffer::<5, f32>::pre_filled(1e-10);
        assert!((buf_small.average() - 1e-10).abs() < MARGIN);

        // Test with negative values
        let mut buf_neg = AvgBuffer::<5, f32>::pre_filled(-5.0);
        assert!((buf_neg.average() - (-5.0)).abs() < MARGIN);
        assert!((buf_neg.min() - (-5.0)).abs() < MARGIN);
        assert!((buf_neg.max() - (-5.0)).abs() < MARGIN);
    }

    #[test]
    fn test_single_element_buffer() {
        let mut buf = AvgBuffer::<1, i32>::new();

        // Push and check
        buf.push(42);
        assert_eq!(buf.average(), 42);
        assert_eq!(buf.min(), 42);
        assert_eq!(buf.max(), 42);

        // Overwrite and check
        buf.push(17);
        assert_eq!(buf.average(), 17);
        assert_eq!(buf.min(), 17);
        assert_eq!(buf.max(), 17);

        // Check iteration
        let mut has_value = false;

        for &val in buf.iter() {
            assert_eq!(val, 17);
            has_value = true;
        }

        assert!(has_value);
    }

    #[test]
    #[should_panic(expected = "Capacity must be larger than zero")]
    fn test_zero_capacity() {
        let _buf = AvgBuffer::<0, f32>::new();
        // This should panic with the message in the attribute
    }

    #[test]
    fn test_basic() {
        let mut buf = AvgBuffer::<5, f32>::new();

        // Push known values: 1, 2, 3, 4, 5
        for i in 1..=5 {
            buf.push(i as f32);
        }

        // Mean should be 3.0
        assert!((buf.average() - 3.0).abs() < MARGIN);
    }

    #[test]
    fn test_rms_and_sum_functions() {
        let mut buf = AvgBuffer::<3, f32>::new();

        buf.push(3.0);
        buf.push(4.0);
        buf.push(5.0);

        // Sum should be 12.0
        assert!((buf.sum() - 12.0).abs() < MARGIN);
    }

    #[test]
    fn test_statistical_functions() {
        let mut buf = AvgBuffer::<5, f32>::new();

        for i in 1..=5 {
            buf.push(i as f32);
        }

        assert!((buf.average() - 3.0).abs() < MARGIN);
        assert!((buf.sum() - 15.0).abs() < MARGIN);
        assert!((buf.range() - 4.0).abs() < MARGIN);
        assert!((buf.approximate_median() - 3.0).abs() < MARGIN);
        assert!(buf.is_trending_up());
    }

    #[test]
    fn test_rolling_updates() {
        let mut buf = AvgBuffer::<3, f32>::new();

        buf.push(1.0);
        buf.push(2.0);
        buf.push(3.0);

        buf.push(10.0); // Overwrites 1.0, buffer now [2, 3, 10]

        assert!((buf.sum() - 15.0).abs() < MARGIN);
        assert!((buf.average() - 5.0).abs() < MARGIN);
    }
}
