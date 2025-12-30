use core::mem::{MaybeUninit, size_of, align_of};
use core::ptr;
use core::slice::Iter;

use crate::{ArenaErr, ArenaId, ArenaIndex, ArenaRes, Slice};

#[derive(Debug, Clone, Copy)]
pub struct RawAllocId<I> {
    pub offset: I,
    pub size: I,
}

/// Trait providing all arena operations.
/// Both Arena and ArenaRef implement this trait.
pub trait ArenaOps<I, M>
where
    I: ArenaIndex,
{
    // Abstract getters that must be implemented
    fn storage_ptr(&self) -> *mut u8;
    fn offset_ref(&self) -> &I;
    fn offset_mut(&mut self) -> &mut I;
    fn tail_offset_ref(&self) -> &I;
    fn tail_offset_mut(&mut self) -> &mut I;
    fn generation_ref(&self) -> &u32;
    fn generation_mut(&mut self) -> &mut u32;
    fn arena_id_ref(&self) -> &u16;
    fn last_alloc_ref(&self) -> &Option<RawAllocId<I>>;
    fn last_alloc_mut(&mut self) -> &mut Option<RawAllocId<I>>;
    fn capacity_bytes(&self) -> usize;
    
    // All arena operations with default implementations
    
    /// Allocate space for a value of type T.
    #[inline]
    fn alloc<T>(&mut self, value: T) -> ArenaRes<ArenaId<T, I, M>> {
        let size = size_of::<T>();
        let align = align_of::<T>();
        
        let offset = *self.offset_ref();
        let aligned_offset = ((offset.to_usize() + align - 1) / align) * align;
        let padding = aligned_offset - offset.to_usize();
        let aligned_offset_i = I::try_from(aligned_offset)
            .map_err(|_| ArenaErr::IndexConversion)?;
        
        let tail_offset = *self.tail_offset_ref();
        let total_size = I::try_from(size + padding)
            .map_err(|_| ArenaErr::IndexConversion)?;
        
        if aligned_offset_i + total_size > tail_offset {
            return Err(ArenaErr::OutOfSpace {
                requested: size,
                available: (tail_offset - aligned_offset_i).to_usize(),
            });
        }
        
        unsafe {
            let ptr = self.storage_ptr().add(aligned_offset) as *mut T;
            ptr.write(value);
        }
        
        *self.offset_mut() = aligned_offset_i + I::try_from(size).unwrap_or_else(|_| panic!("size too large for I"));
        *self.last_alloc_mut() = Some(RawAllocId {
            offset: aligned_offset_i,
            size: I::try_from(size).unwrap_or_else(|_| panic!("size too large for I")),
        });
        
        Ok(ArenaId::new(
            aligned_offset_i,
            I::try_from(size).unwrap_or_else(|_| panic!("size too large for I")),
            *self.generation_ref(),
            *self.arena_id_ref(),
        ))
    }
    
    /// Allocate a slice using a generator function.
    #[inline]
    fn alloc_slice_from_fn<T, F>(&mut self, len: usize, mut f: F) -> ArenaRes<Slice<T, I, M>>
    where
        F: FnMut(usize) -> T,
    {
        if len == 0 {
            return Ok(Slice::default());
        }
        
        let item_size = size_of::<T>();
        let align = align_of::<T>();
        let total_size = item_size * len;
        
        let offset = *self.offset_ref();
        let aligned_offset = ((offset.to_usize() + align - 1) / align) * align;
        let padding = aligned_offset - offset.to_usize();
        let aligned_offset_i = I::try_from(aligned_offset)
            .map_err(|_| ArenaErr::IndexConversion)?;
        
        let tail_offset = *self.tail_offset_ref();
        let total_with_padding = I::try_from(total_size + padding)
            .map_err(|_| ArenaErr::IndexConversion)?;
        
        if aligned_offset_i + total_with_padding > tail_offset {
            return Err(ArenaErr::OutOfSpace {
                requested: total_size,
                available: (tail_offset - aligned_offset_i).to_usize(),
            });
        }
        
        unsafe {
            let base_ptr = self.storage_ptr().add(aligned_offset) as *mut T;
            
            for i in 0..len {
                base_ptr.add(i).write(f(i));
            }
        }
        
        *self.offset_mut() = aligned_offset_i + I::try_from(total_size).unwrap_or_else(|_| panic!("total_size too large for I"));
        
        Ok(Slice::new(
            aligned_offset_i,
            I::try_from(len).unwrap_or_else(|_| panic!("len too large for I")),
            *self.generation_ref(),
            *self.arena_id_ref(),
        ))
    }
    
    /// Allocate a slice from a source slice.
    #[inline]
    fn alloc_slice<T: Clone>(&mut self, src: &[T]) -> ArenaRes<Slice<T, I, M>> {
        self.alloc_slice_from_fn(src.len(), |i| src[i].clone())
    }
    
    /// Allocate a slice from an iterator.
    #[inline]
    fn alloc_slice_from_iter<T>(&mut self, iter: impl IntoIterator<Item = T>) -> ArenaRes<Slice<T, I, M>> {
        // Collect items into a temporary buffer
        // For no_std, we need a fixed-size buffer
        let mut items: [MaybeUninit<T>; 1024] = unsafe { MaybeUninit::uninit().assume_init() };
        let mut len = 0;
        
        for (i, item) in iter.into_iter().enumerate() {
            if i >= 1024 {
                return Err(ArenaErr::CapacityExceeded);
            }
            items[i] = MaybeUninit::new(item);
            len = i + 1;
        }
        
        self.alloc_slice_from_fn(len, |i| unsafe {
            items[i].assume_init_read()
        })
    }
    
    /// Allocate uninitialized slice and return it for initialization.
    #[inline]
    fn alloc_slice_uninit<T>(&mut self, len: usize) -> ArenaRes<(Slice<T, I, M>, &mut [MaybeUninit<T>])> {
        if len == 0 {
            return Ok((Slice::default(), &mut []));
        }
        
        let item_size = size_of::<T>();
        let align = align_of::<T>();
        let total_size = item_size * len;
        
        let offset = *self.offset_ref();
        let aligned_offset = ((offset.to_usize() + align - 1) / align) * align;
        let padding = aligned_offset - offset.to_usize();
        let aligned_offset_i = I::try_from(aligned_offset)
            .map_err(|_| ArenaErr::IndexConversion)?;
        
        let tail_offset = *self.tail_offset_ref();
        let total_with_padding = I::try_from(total_size + padding)
            .map_err(|_| ArenaErr::IndexConversion)?;
        
        if aligned_offset_i + total_with_padding > tail_offset {
            return Err(ArenaErr::OutOfSpace {
                requested: total_size,
                available: (tail_offset - aligned_offset_i).to_usize(),
            });
        }
        
        let slice = Slice::new(
            aligned_offset_i,
            I::try_from(len).unwrap_or_else(|_| panic!("len too large for I")),
            *self.generation_ref(),
            *self.arena_id_ref(),
        );
        
        *self.offset_mut() = aligned_offset_i + I::try_from(total_size).unwrap_or_else(|_| panic!("total_size too large for I"));
        
        unsafe {
            let base_ptr = self.storage_ptr().add(aligned_offset) as *mut MaybeUninit<T>;
            let uninit_slice = core::slice::from_raw_parts_mut(base_ptr, len);
            Ok((slice, uninit_slice))
        }
    }
    
    /// Internal: validate an ID
    #[inline]
    fn validate_id<T>(&self, id: &ArenaId<T, I, M>) -> ArenaRes<()> {
        if id.arena_id != *self.arena_id_ref() {
            return Err(ArenaErr::CrossArenaAccess {
                expected_id: *self.arena_id_ref(),
                found_id: id.arena_id,
            });
        }
        
        if id.generation != *self.generation_ref() {
            return Err(ArenaErr::InvalidGeneration {
                expected: *self.generation_ref(),
                found: id.generation,
            });
        }
        
        let end = id.offset.to_usize() + id.size.to_usize();
        if end > self.offset_ref().to_usize() {
            return Err(ArenaErr::InvalidBounds);
        }
        
        Ok(())
    }
    
    /// Get a reference to a value.
    #[inline]
    fn get<T>(&self, id: ArenaId<T, I, M>) -> ArenaRes<&T> {
        self.validate_id(&id)?;
        
        unsafe {
            let ptr = self.storage_ptr().add(id.offset.to_usize()) as *const T;
            Ok(&*ptr)
        }
    }
    
    /// Get a mutable reference to a value.
    #[inline]
    fn get_mut<T>(&mut self, id: ArenaId<T, I, M>) -> ArenaRes<&mut T> {
        self.validate_id(&id)?;
        
        unsafe {
            let ptr = self.storage_ptr().add(id.offset.to_usize()) as *mut T;
            Ok(&mut *ptr)
        }
    }
    
    /// Internal: validate a slice
    #[inline]
    fn validate_slice<T>(&self, slice: &Slice<T, I, M>) -> ArenaRes<()> {
        if slice.is_empty() {
            return Ok(());
        }
        
        if slice.arena_id != *self.arena_id_ref() {
            return Err(ArenaErr::CrossArenaAccess {
                expected_id: *self.arena_id_ref(),
                found_id: slice.arena_id,
            });
        }
        
        if slice.generation != *self.generation_ref() {
            return Err(ArenaErr::InvalidGeneration {
                expected: *self.generation_ref(),
                found: slice.generation,
            });
        }
        
        let end = slice.offset.to_usize() + (slice.len.to_usize() * size_of::<T>());
        if end > self.offset_ref().to_usize() {
            return Err(ArenaErr::InvalidBounds);
        }
        
        Ok(())
    }
    
    /// Get a slice reference.
    #[inline]
    fn get_slice<T>(&self, slice: Slice<T, I, M>) -> ArenaRes<&[T]> {
        self.validate_slice(&slice)?;
        
        if slice.is_empty() {
            return Ok(&[]);
        }
        
        unsafe {
            let ptr = self.storage_ptr().add(slice.offset.to_usize()) as *const T;
            Ok(core::slice::from_raw_parts(ptr, slice.len.to_usize()))
        }
    }
    
    /// Get a mutable slice reference.
    #[inline]
    fn get_slice_mut<T>(&mut self, slice: Slice<T, I, M>) -> ArenaRes<&mut [T]> {
        self.validate_slice(&slice)?;
        
        if slice.is_empty() {
            return Ok(&mut []);
        }
        
        unsafe {
            let ptr = self.storage_ptr().add(slice.offset.to_usize()) as *mut T;
            Ok(core::slice::from_raw_parts_mut(ptr, slice.len.to_usize()))
        }
    }
    
    /// Clear the arena, resetting all allocations.
    #[inline]
    fn clear(&mut self) {
        *self.offset_mut() = I::zero();
        *self.tail_offset_mut() = I::try_from(self.capacity_bytes())
            .unwrap_or_else(|_| panic!("capacity too large for I"));
        *self.generation_mut() = self.generation_ref().wrapping_add(1);
        *self.last_alloc_mut() = None;
    }
    
    /// Get the amount of space used.
    #[inline]
    fn used(&self) -> usize {
        let offset = self.offset_ref().to_usize();
        let tail_used = self.capacity_bytes() - self.tail_offset_ref().to_usize();
        offset + tail_used
    }
    
    /// Get the remaining space.
    #[inline]
    fn remaining(&self) -> usize {
        self.tail_offset_ref().to_usize() - self.offset_ref().to_usize()
    }
    
    /// Get the total capacity.
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity_bytes()
    }
    
    /// Get the current generation.
    #[inline]
    fn generation(&self) -> u32 {
        *self.generation_ref()
    }
    
    /// Restore the arena to a previous state.
    #[inline]
    fn restore_to(&mut self, offset: usize) {
        *self.offset_mut() = I::try_from(offset)
            .unwrap_or_else(|_| panic!("offset too large for I"));
        *self.generation_mut() = self.generation_ref().wrapping_add(1);
        *self.last_alloc_mut() = None;
    }
    
    /// Get the arena ID.
    #[inline]
    fn arena_id(&self) -> u16 {
        *self.arena_id_ref()
    }
    
    /// Pop the last allocation if it matches the expected type.
    #[inline]
    fn pop<T>(&mut self) -> ArenaRes<T> {
        if let Some(last) = *self.last_alloc_ref() {
            if last.size.to_usize() == size_of::<T>() {
                unsafe {
                    let ptr = self.storage_ptr().add(last.offset.to_usize()) as *mut T;
                    let value = ptr::read(ptr);
                    
                    *self.offset_mut() = last.offset;
                    *self.last_alloc_mut() = None;
                    
                    Ok(value)
                }
            } else {
                Err(ArenaErr::InvalidBounds)
            }
        } else {
            Err(ArenaErr::InvalidBounds)
        }
    }
    
    /// Iterate over a slice.
    #[inline]
    fn iter_slice<T>(&self, slice: Slice<T, I, M>) -> ArenaRes<Iter<'_, T>> {
        Ok(self.get_slice(slice)?.iter())
    }
    
    /// Iterate over a range of a slice.
    #[inline]
    fn iter_slice_range<T>(&self, slice: Slice<T, I, M>, start: usize, end: usize) -> ArenaRes<Iter<'_, T>> {
        if start > end || end > slice.len.to_usize() {
            return Err(ArenaErr::InvalidBounds);
        }
        
        if slice.is_empty() || start == end {
            return Ok([].iter());
        }
        
        self.validate_slice(&slice)?;
        
        unsafe {
            let ptr = self.storage_ptr().add(slice.offset.to_usize()) as *const T;
            let sub_slice = core::slice::from_raw_parts(ptr.add(start), end - start);
            Ok(sub_slice.iter())
        }
    }
}