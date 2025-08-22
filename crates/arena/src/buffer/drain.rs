use super::*;

pub struct DrainIterator<'a, T, const LEN: usize, Idx = u16, Marker = ()> {
    pub(super) arena: &'a Arena<LEN, Idx, Marker>,
    pub(super)slice: Slice<T, Idx, Marker>,
    pub(super)current: usize,
    pub(super)end: usize,
}

impl<'a, T, const LEN: usize, Idx, Marker> Iterator for DrainIterator<'a, T, LEN, Idx, Marker>
where
    Idx: ArenaIndex,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.end {
            return None;
        }

        let slice = self.arena.get_slice(&self.slice).ok()?;
        let value = unsafe { core::ptr::read(&slice[self.current]) };
        self.current += 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.end - self.current;
        (remaining, Some(remaining))
    }
}

impl<'a, T, const LEN: usize, Idx, Marker> ExactSizeIterator for DrainIterator<'a, T, LEN, Idx, Marker>
where
    Idx: ArenaIndex,
{
    fn len(&self) -> usize {
        self.end - self.current
    }
}
