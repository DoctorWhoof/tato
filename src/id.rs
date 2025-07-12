//! Arena ID - Handle to values allocated in the arena.

use core::marker::PhantomData;

/// A type-erased handle to an allocated value in the arena
///
/// This can represent any allocation regardless of type, making it useful
/// for storing mixed types in collections. Use `typed()` to convert back
/// to a typed `ArenaId` when you know the type.
///
/// # Examples
///
/// ```
/// # use tato_arena::{Arena, RawId};
/// # struct Tilemap<const N: usize> { data: [u8; N] }
/// let mut arena: Arena<1024> = Arena::new();
///
/// // Allocate different types
/// let id1 = arena.alloc(42u32).unwrap();
/// let id2 = arena.alloc("hello").unwrap();
/// let id3 = arena.alloc(Tilemap::<64> { data: [0; 64] }).unwrap();
///
/// // Convert to RawId for unified storage
/// let raw_ids = [id1.raw(), id2.raw(), id3.raw()];
///
/// // Later, convert back when you know the type
/// let value: u32 = *arena.get(&raw_ids[0].typed());
/// assert_eq!(value, 42);
/// ```
///
/// # Type Safety
///
/// In debug builds, converting to the wrong type will panic:
///
/// ```should_panic
/// # use tato_arena::{Arena, ArenaId, RawId};
/// # let mut arena: Arena<1024> = Arena::new();
/// let id = arena.alloc(42u32).unwrap();  // 4 bytes
/// let raw = id.raw();
///
/// // This will panic in debug mode (u64 is 8 bytes, not 4)
/// let wrong: ArenaId<u64> = raw.typed();
/// ```
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct RawId<SizeType = usize> {
    /// Offset within the arena's storage
    pub(crate) offset: SizeType,
    /// Size of the allocation in bytes
    pub(crate) size: SizeType,
    /// Size of the original type in bytes (for type checking)
    pub(crate) type_size: SizeType,
}

impl<SizeType> RawId<SizeType>
where
    SizeType: Copy + PartialEq + TryFrom<usize> + Into<usize>,
{
    /// Convert to a typed ID when you know the type
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the size of type T doesn't match the stored type_size.
    /// This helps catch type confusion errors.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tato_arena::{Arena, RawId};
    /// let mut arena: Arena<1024> = Arena::new();
    /// let id = arena.alloc([1.0, 2.0, 3.0]).unwrap();
    /// let raw = id.raw();
    ///
    /// // Convert back to the correct type
    /// let typed_id = raw.typed::<[f64; 3]>();
    /// let array = arena.get(&typed_id);
    /// assert_eq!(array[0], 1.0);
    /// ```
    pub fn typed<T>(self) -> ArenaId<T, SizeType> {
        let expected_size = core::mem::size_of::<T>();
        let stored_size: usize = self.type_size.into();
        debug_assert_eq!(
            stored_size,
            expected_size,
            "Type size mismatch: attempted to convert RawId to wrong type. \
             Expected size: {}, but T has size: {}",
            stored_size,
            expected_size
        );

        ArenaId {
            offset: self.offset,
            size: self.size,
            _marker: PhantomData,
        }
    }
}

/// A handle to an allocated value in the arena
#[derive(Debug, Clone, Copy, Hash)]
pub struct ArenaId<T, SizeType = usize> {
    /// Offset within the arena's storage
    pub(crate) offset: SizeType,
    /// Size of the allocation in bytes
    pub(crate) size: SizeType,
    /// Zero-sized type marker for compile-time type safety
    pub(crate) _marker: PhantomData<T>,
}

impl<T, SizeType> ArenaId<T, SizeType> {
    /// Create a new ArenaId with the given offset and size
    ///
    /// This is primarily used internally by the arena allocator.
    pub(crate) fn new(offset: SizeType, size: SizeType) -> Self {
        Self {
            offset,
            size,
            _marker: PhantomData,
        }
    }

    /// Get the offset of this allocation within the arena
    ///
    /// This is the byte offset from the start of the arena's storage
    /// where this value is located.
    pub fn offset(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.offset.into()
    }

    /// Get the size of this allocation in bytes
    ///
    /// This is the actual size of the allocated type T.
    pub fn size(&self) -> usize
    where
        SizeType: Copy + Into<usize>,
    {
        self.size.into()
    }

    /// Get the type information as a tuple (offset, size)
    ///
    /// Useful for debugging and introspection.
    pub fn info(&self) -> (usize, usize)
    where
        SizeType: Copy + Into<usize>,
    {
        (self.offset.into(), self.size.into())
    }

    /// Check if this ID represents a valid allocation
    ///
    /// An ID is considered valid if it has non-zero size.
    /// Note: This doesn't check if the arena is still alive.
    pub fn is_valid(&self) -> bool
    where
        SizeType: Copy + Into<usize>,
    {
        self.size.into() > 0
    }

    /// Convert this typed ID to a type-erased RawId
    ///
    /// This is useful when you need to store IDs of different types together.
    /// The type size is preserved for safety checks when converting back.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tato_arena::{Arena, ArenaId, RawId};
    /// # struct Player { x: f32, y: f32 }
    /// # struct Enemy { health: u32 }
    /// let mut arena: Arena<1024> = Arena::new();
    ///
    /// // Different entity types
    /// let player_id = arena.alloc(Player { x: 0.0, y: 0.0 }).unwrap();
    /// let enemy_id = arena.alloc(Enemy { health: 100 }).unwrap();
    ///
    /// // Store both in the same collection
    /// let entities: [RawId; 2] = [player_id.raw(), enemy_id.raw()];
    /// ```
    pub fn raw(self) -> RawId<SizeType>
    where
        SizeType: TryFrom<usize>,
    {
        RawId {
            offset: self.offset,
            size: self.size,
            type_size: SizeType::try_from(core::mem::size_of::<T>())
                .unwrap_or_else(|_| panic!("Type size too large for SizeType")),
        }
    }
}

/// Equality comparison for ArenaId
///
/// Two ArenaIds are equal if they point to the same location
/// and have the same size. The type is enforced at compile time.
impl<T, SizeType> PartialEq for ArenaId<T, SizeType>
where
    SizeType: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.size == other.size
    }
}

impl<T, SizeType> Eq for ArenaId<T, SizeType>
where
    SizeType: Eq,
{}

// /// Hash implementation for ArenaId
// ///
// /// Allows ArenaId to be used as keys in hash maps/sets.
// impl<T, SizeType> core::hash::Hash for ArenaId<T, SizeType>
// where
//     SizeType: core::hash::Hash,
// {
//     fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
//         self.offset.hash(state);
//         self.size.hash(state);
//     }
// }
