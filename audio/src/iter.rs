use crate::*;

/// Iterates a specified number of samples. Use [AudioChip::iter()] to obtain this.
pub struct SoundChipIter<'a> {
    chip: &'a mut AudioChip,
    head: usize,
    sample_count: usize,
}

impl<'a> Iterator for SoundChipIter<'a> {
    type Item = Sample<i16>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head < self.sample_count {
            self.head += 1;
            return Some(self.chip.process_sample());
        }
        None
    }
}
