use core::marker::PhantomData;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU16, Ordering};

use crate::{ArenaIndex, ArenaOps, RawAllocId};

// Global counter for unique arena IDs (no-std compatible)
static ARENA_ID_COUNTER: AtomicU16 = AtomicU16::new(1);

/// Fixed-size arena with generational safety.
/// LEN = bytes, I = handle size, M = type safety marker.
#[repr(C, align(16))]
#[derive(Debug)]
pub struct Arena<const LEN: usize, I = u32, M = ()> {
    /// Raw storage for all allocations
    pub(crate) storage: [MaybeUninit<u8>; LEN],
    /// Current allocation offset (bump pointer)
    pub(crate) offset: I,
    /// Current tail allocation offset (allocates backwards from end)
    pub(crate) tail_offset: I,
    /// Current generation - incremented on restore_to()
    pub(crate) generation: u32,
    /// Unique arena ID for cross-arena safety
    pub(crate) arena_id: u16,
    /// Last allocation for pop() support
    pub(crate) last_alloc: Option<RawAllocId<I>>,
    /// Zero-sized type marker for compile-time arena safety
    pub(crate) _marker: PhantomData<M>,
}

impl<const LEN: usize, I, M> Arena<LEN, I, M>
where
    I: ArenaIndex,
{
    /// Create a new arena with automatic cross-arena safety.
    /// Each arena gets a unique ID from an atomic counter, ensuring
    /// collision resistance and automatic cross-arena safety without requiring
    /// explicit marker types.
    pub fn new() -> Self {
        let storage = unsafe { MaybeUninit::uninit().assume_init() };
        Self {
            storage,
            offset: I::try_from(0).unwrap_or_else(|_| panic!("I too small")),
            tail_offset: I::try_from(LEN).unwrap_or_else(|_| panic!("I too small for LEN")),
            generation: 0,
            arena_id: ARENA_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            last_alloc: None,
            _marker: PhantomData,
        }
    }

    // Tail allocation methods have been moved to ArenaOps trait as internal methods
}

// Implement the ArenaOps trait for Arena
impl<const LEN: usize, I, M> ArenaOps<I, M> for Arena<LEN, I, M>
where
    I: ArenaIndex,
{
    #[inline(always)]
    fn storage_ptr(&self) -> *mut u8 {
        self.storage.as_ptr() as *mut u8
    }

    #[inline(always)]
    fn offset_ref(&self) -> &I {
        &self.offset
    }

    #[inline(always)]
    fn offset_mut(&mut self) -> &mut I {
        &mut self.offset
    }

    #[inline(always)]
    fn tail_offset_ref(&self) -> &I {
        &self.tail_offset
    }

    #[inline(always)]
    fn tail_offset_mut(&mut self) -> &mut I {
        &mut self.tail_offset
    }

    #[inline(always)]
    fn generation_ref(&self) -> &u32 {
        &self.generation
    }

    #[inline(always)]
    fn generation_mut(&mut self) -> &mut u32 {
        &mut self.generation
    }

    #[inline(always)]
    fn arena_id_ref(&self) -> &u16 {
        &self.arena_id
    }

    #[inline(always)]
    fn last_alloc_ref(&self) -> &Option<RawAllocId<I>> {
        &self.last_alloc
    }

    #[inline(always)]
    fn last_alloc_mut(&mut self) -> &mut Option<RawAllocId<I>> {
        &mut self.last_alloc
    }

    #[inline(always)]
    fn capacity_bytes(&self) -> usize {
        LEN
    }
}

impl<const LEN: usize, I, M> Default for Arena<LEN, I, M>
where
    I: ArenaIndex,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArenaOps;

    #[test]
    fn test_generation_wraparound() {
        let mut arena = Arena::<1024, u32>::new();

        // Set generation to max value - 1
        *arena.generation_mut() = u32::MAX - 1;

        // Allocate something
        let id1 = arena.alloc(42u32).unwrap();
        assert_eq!(id1.generation, u32::MAX - 1);

        // Clear (increments generation)
        arena.clear();
        assert_eq!(arena.generation(), u32::MAX);

        // Clear again (should wrap to 0)
        arena.clear();
        assert_eq!(arena.generation(), 0);

        // Allocate with wrapped generation
        let id2 = arena.alloc(43u32).unwrap();
        assert_eq!(id2.generation, 0);
    }
}
