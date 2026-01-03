use super::*;
use crate::ArenaOps;
use core::marker::PhantomData;

pub struct RingBufferIterator<'a, A, T, I, M>
where
    I: ArenaIndex,
    A: ArenaOps<I, M>,
{
    arena: &'a A,
    slice_offset: I,
    slice_len: I,
    slice_generation: u32,
    slice_arena_id: u16,
    head: I,
    current: usize,
    back: usize,
    _phantom_t: PhantomData<&'a T>,
    _phantom_m: PhantomData<&'a M>,
}

impl<'a, A, T, I, M> RingBufferIterator<'a, A, T, I, M>
where
    I: ArenaIndex,
    A: ArenaOps<I, M>,
{
    pub(super) fn new(arena: &'a A, slice: &Slice<T, I, M>, head: I, len: I) -> Self {
        Self {
            arena,
            slice_offset: slice.offset(),
            slice_len: slice.len(),
            slice_generation: slice.generation(),
            slice_arena_id: slice.arena_id(),
            head,
            current: 0,
            back: len.to_usize(),
            _phantom_t: PhantomData,
            _phantom_m: PhantomData,
        }
    }
}

impl<'a, A, T, I, M> Iterator for RingBufferIterator<'a, A, T, I, M>
where
    T: 'a,
    I: ArenaIndex,
    A: ArenaOps<I, M>,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.back {
            return None;
        }

        let slice_ref = Slice::new(
            self.slice_offset,
            self.slice_len,
            self.slice_generation,
            self.slice_arena_id,
        );
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

impl<'a, A, T: 'a, I, M> ExactSizeIterator for RingBufferIterator<'a, A, T, I, M>
where
    I: ArenaIndex,
    A: ArenaOps<I, M>,
{
    fn len(&self) -> usize {
        self.back - self.current
    }
}

impl<'a, A, T: 'a, I, M> DoubleEndedIterator for RingBufferIterator<'a, A, T, I, M>
where
    I: ArenaIndex,
    A: ArenaOps<I, M>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current >= self.back {
            return None;
        }

        self.back -= 1;

        let slice_ref = Slice::new(
            self.slice_offset,
            self.slice_len,
            self.slice_generation,
            self.slice_arena_id,
        );
        let slice = self.arena.get_slice(slice_ref).ok()?;
        let capacity = self.slice_len.to_usize();
        let physical_index = (self.head.to_usize() + self.back) % capacity;

        Some(&slice[physical_index])
    }
}
