use super::*;
use crate::ArenaOps;

pub struct DrainIterator<'a, T, const LEN: usize, I = u32, M = ()> {
    pub(super) arena: &'a Arena<LEN, I, M>,
    pub(super)slice: Slice<T, I, M>,
    pub(super)current: usize,
    pub(super)end: usize,
}

impl<'a, T, const LEN: usize, I, M> Iterator for DrainIterator<'a, T, LEN, I, M>
where
    I: ArenaIndex,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let slice = self.arena.get_slice(self.slice.clone()).ok()?;
        let value = unsafe { core::ptr::read(&slice[self.current]) };
        self.current += 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end - self.current;
        (remaining, Some(remaining))
    }
}

impl<'a, T, const LEN: usize, I, M> ExactSizeIterator for DrainIterator<'a, T, LEN, I, M>
where
    I: ArenaIndex,
{
    fn len(&self) -> usize {
        self.end - self.current
    }
}
