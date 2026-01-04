use core::marker::PhantomData;

use crate::{Arena, ArenaIndex, ArenaOps, RawAllocId};

/// Size-erased reference to an Arena.
/// Zero-cost abstraction that removes the const LEN parameter.
pub struct ArenaRef<'a, I = u32, M = ()> {
    storage: *mut u8,
    offset: *mut I,
    tail_offset: *mut I,
    generation: *mut u32,
    arena_id: *const u16,
    last_alloc: *mut Option<RawAllocId<I>>,
    capacity: usize,
    _phantom: PhantomData<(&'a mut (), I, M)>,
}

// Safety: ArenaRef is Send/Sync if the underlying types are
unsafe impl<'a, I: Send, M> Send for ArenaRef<'a, I, M> {}
unsafe impl<'a, I: Sync, M> Sync for ArenaRef<'a, I, M> {}

impl<const LEN: usize, I, M> Arena<LEN, I, M>
where
    I: ArenaIndex,
{
    /// Create a size-erased reference to this arena.
    #[inline]
    pub fn as_ref(&mut self) -> ArenaRef<'_, I, M> {
        ArenaRef {
            storage: self.storage.as_mut_ptr() as *mut u8,
            offset: &mut self.offset as *mut I,
            tail_offset: &mut self.tail_offset as *mut I,
            generation: &mut self.generation as *mut u32,
            arena_id: &self.arena_id as *const u16,
            last_alloc: &mut self.last_alloc as *mut Option<RawAllocId<I>>,
            capacity: LEN,
            _phantom: PhantomData,
        }
    }
}

impl<'a, I, M> ArenaOps<I, M> for ArenaRef<'a, I, M>
where
    I: ArenaIndex,
{
    #[inline(always)]
    fn storage_ptr(&self) -> *mut u8 {
        self.storage
    }
    
    #[inline(always)]
    fn offset_ref(&self) -> &I {
        unsafe { &*self.offset }
    }
    
    #[inline(always)]
    fn offset_mut(&mut self) -> &mut I {
        unsafe { &mut *self.offset }
    }
    
    #[inline(always)]
    fn tail_offset_ref(&self) -> &I {
        unsafe { &*self.tail_offset }
    }
    
    #[inline(always)]
    fn tail_offset_mut(&mut self) -> &mut I {
        unsafe { &mut *self.tail_offset }
    }
    
    #[inline(always)]
    fn generation_ref(&self) -> &u32 {
        unsafe { &*self.generation }
    }
    
    #[inline(always)]
    fn generation_mut(&mut self) -> &mut u32 {
        unsafe { &mut *self.generation }
    }
    
    #[inline(always)]
    fn arena_id_ref(&self) -> &u16 {
        unsafe { &*self.arena_id }
    }
    
    #[inline(always)]
    fn last_alloc_ref(&self) -> &Option<RawAllocId<I>> {
        unsafe { &*self.last_alloc }
    }
    
    #[inline(always)]
    fn last_alloc_mut(&mut self) -> &mut Option<RawAllocId<I>> {
        unsafe { &mut *self.last_alloc }
    }
    
    #[inline(always)]
    fn capacity_bytes(&self) -> usize {
        self.capacity
    }
}