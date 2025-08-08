use super::*;

#[derive(Debug)]
pub struct FillablePool<T, SizeType> {
    pool: Pool<T, SizeType>,
    len: usize,      // Current number of elements used
    capacity: usize, // Maximum elements (from original allocation)
}

impl<T, SizeType: ArenaIndex> FillablePool<T, SizeType> {
    pub fn new<const LEN: usize>(
        arena: &mut Arena<LEN, SizeType>,
        capacity: usize,
    ) -> Option<Self>
    where
        T: Default,
    {
        let pool = arena.alloc_pool::<T>(capacity)?;
        Some(Self { pool, len: 0, capacity })
    }

    pub fn push<const LEN:usize>(&mut self, arena: &mut Arena<LEN, SizeType>, value: T) -> Result<(), T> {
        if self.len >= self.capacity {
            return Err(value); // Return the value back if full
        }

        let slice = match arena.get_pool_mut(&self.pool) {
            Some(slice) => slice,
            None => return Err(value),
        };
        slice[self.len] = value;
        self.len += 1;
        Ok(())
    }

    pub fn as_slice<'a, const LEN:usize>(&self, arena: &'a Arena<LEN, SizeType>) -> Option<&'a [T]> {
        let full_slice = arena.get_pool(&self.pool)?;
        Some(&full_slice[..self.len])
    }
}
