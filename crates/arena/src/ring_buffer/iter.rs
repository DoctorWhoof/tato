use super::*;
use crate::ArenaOps;
use core::marker::PhantomData;

pub struct RingBufferIterator<'a, T, const LEN: usize, I, M> {
    arena: &'a Arena<LEN, I, M>,
    slice_offset: I,
    slice_len: I,
    slice_generation: u32,
    slice_arena_id: u16,
    head: I,
    current: usize,
    back: usize,
    _phantom: PhantomData<&'a T>,
}

impl<'a, T, const LEN: usize, I, M> RingBufferIterator<'a, T, LEN, I, M>
where
    I: ArenaIndex,
{
    pub(super) fn new(
        arena: &'a Arena<LEN, I, M>,
        slice: &Slice<T, I, M>,
        head: I,
        len: I,
    ) -> Self {
        Self {
            arena,
            slice_offset: slice.offset(),
            slice_len: slice.len(),
            slice_generation: slice.generation(),
            slice_arena_id: slice.arena_id(),
            head,
            current: 0,
            back: len.to_usize(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, T: 'a, const LEN: usize, I, M> Iterator for RingBufferIterator<'a, T, LEN, I, M>
where
    I: ArenaIndex,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.back {
            return None;
        }

        let slice_ref = Slice::new(self.slice_offset, self.slice_len, self.slice_generation, self.slice_arena_id);
        let slice = self.arena.get_slice(slice_ref).ok()?;
        let capacity = self.slice_len.to_usize();
        let physical_index = (self.head.to_usize() + self.current) % capacity;

        self.current += 1;
        Some(&slice[physical_index])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.back - self.current;
        (remaining, Some(remaining))
    }
}

impl<'a, T: 'a, const LEN: usize, I, M> ExactSizeIterator for RingBufferIterator<'a, T, LEN, I, M>
where
    I: ArenaIndex,
{
    fn len(&self) -> usize {
        self.back - self.current
    }
}

impl<'a, T: 'a, const LEN: usize, I, M> DoubleEndedIterator for RingBufferIterator<'a, T, LEN, I, M>
where
    I: ArenaIndex,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current >= self.back {
            return None;
        }

        self.back -= 1;

        let slice_ref = Slice::new(self.slice_offset, self.slice_len, self.slice_generation, self.slice_arena_id);
        let slice = self.arena.get_slice(slice_ref).ok()?;
        let capacity = self.slice_len.to_usize();
        let physical_index = (self.head.to_usize() + self.back) % capacity;

        Some(&slice[physical_index])
    }
}
