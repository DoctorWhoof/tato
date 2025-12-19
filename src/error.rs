use core::fmt;
use tato_arena::ArenaErr;

/// Convenience type alias (lets you omit "TatoError", so you can return
/// "TatoResult<T>" instead of "Result<T, TatoError>")
pub type TatoResult<T> = Result<T, TatoError>;

/// Error type for asset system operations
#[derive(Debug, Clone, PartialEq)]
pub enum TatoError {
    /// Invalid bank ID provided
    InvalidBankId(u8),
    /// Tileset capacity exceeded in bank
    TilesetCapacityExceeded { bank_id: u8, requested: usize, available: usize },
    /// Checkpoint stack overflow (max 32 tilesets)
    CheckpointStackOverflow,
    /// No tileset available to pop
    NoTilesetToPop,
    /// Invalid tileset ID
    InvalidTilesetId(u8),
    /// Can only load maps for the current (most recent) tileset
    InvalidTilesetForMap { provided: u8, expected: u8 },
    /// Map capacity exceeded
    MapCapacityExceeded { bank_id: u8 },
    /// Tilemap capacity reached (255 max)
    TilemapCapacityReached,
    /// Invalid tilemap dimensions
    InvalidTilemapDimensions { len: usize, columns: u16 },
    /// Arena operation failed
    Arena(ArenaErr),
    /// Animation frames capacity exceeded
    AnimationFramesCapacityExceeded,
    /// Not enough space for animation frames
    InsufficientAnimationFrameSpace { requested: usize, available: u8 },
    /// Animation capacity exceeded
    AnimationCapacityExceeded,
    /// Animation capacity reached (255 max)
    AnimationCapacityReached,
    /// Invalid animation ID
    InvalidAnimId(u8),
    /// Invalid strip ID
    InvalidStripId(u8),
    /// Invalid frame index for animation
    InvalidFrameIndex { frame: u8, max_frames: u8 },
    /// Invalid map ID
    InvalidMapId(u8),
    /// Frame count exceeds u8 limit
    FrameCountTooLarge(usize),

}

impl core::error::Error for TatoError {}

impl fmt::Display for TatoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TatoError::InvalidBankId(id) => write!(f, "Invalid bank ID: {}", id),
            TatoError::TilesetCapacityExceeded { bank_id, requested, available } => {
                write!(
                    f,
                    "Tileset capacity exceeded in bank {}: requested {}, available {}",
                    bank_id, requested, available
                )
            },
            TatoError::CheckpointStackOverflow => {
                write!(f, "Checkpoint stack overflow (max 32 tilesets)")
            },
            TatoError::NoTilesetToPop => write!(f, "No tileset to pop"),
            TatoError::InvalidTilesetId(id) => write!(f, "Invalid tileset ID: {}", id),
            TatoError::InvalidTilesetForMap { provided, expected } => {
                write!(
                    f,
                    "Can only load maps for the current tileset: provided {}, expected {}",
                    provided, expected
                )
            },
            TatoError::MapCapacityExceeded { bank_id } => {
                write!(f, "Map capacity exceeded on bank {}", bank_id)
            },
            TatoError::TilemapCapacityReached => write!(f, "Tilemap capacity reached"),
            TatoError::InvalidTilemapDimensions { len, columns } => {
                write!(
                    f,
                    "Invalid tilemap dimensions: data.len() ({}) must be divisible by columns ({})",
                    len, columns
                )
            },
            TatoError::Arena(arena_error) => write!(f, "Arena error: {}", arena_error),
            TatoError::AnimationFramesCapacityExceeded => {
                write!(f, "Animation frames capacity exceeded")
            },
            TatoError::InsufficientAnimationFrameSpace { requested, available } => {
                write!(
                    f,
                    "Not enough space to fit {} animation frames, only {} left",
                    requested, available
                )
            },
            TatoError::AnimationCapacityExceeded => write!(f, "Animation capacity exceeded"),
            TatoError::AnimationCapacityReached => write!(f, "Animation capacity reached"),
            TatoError::InvalidAnimId(id) => write!(f, "Invalid animation ID: {}", id),
            TatoError::InvalidStripId(id) => write!(f, "Invalid strip ID: {}", id),
            TatoError::InvalidFrameIndex { frame, max_frames } => {
                write!(f, "Invalid frame index {} (max frames: {})", frame, max_frames)
            },
            TatoError::InvalidMapId(id) => write!(f, "Invalid map ID: {}", id),
            TatoError::FrameCountTooLarge(count) => {
                write!(f, "Frame count {} exceeds u8 limit", count)
            },

        }
    }
}

impl From<ArenaErr> for TatoError {
    fn from(value: ArenaErr) -> Self {
        TatoError::Arena(value)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_error_payload() {
        // Test that TatoError::Arena can wrap ArenaErr
        let arena_error = ArenaErr::OutOfSpace { requested: 256, available: 128 };
        let tato_error = TatoError::Arena(arena_error);

        // Verify we can match on the wrapped error
        match tato_error {
            TatoError::Arena(ArenaErr::OutOfSpace { requested, available }) => {
                assert_eq!(requested, 256);
                assert_eq!(available, 128);
            }
            _ => panic!("Expected Arena error with OutOfSpace"),
        }

        // Test generation mismatch error
        let gen_error = ArenaErr::InvalidGeneration { expected: 5, found: 3 };
        let tato_gen_error = TatoError::Arena(gen_error);

        match tato_gen_error {
            TatoError::Arena(ArenaErr::InvalidGeneration { expected, found }) => {
                assert_eq!(expected, 5);
                assert_eq!(found, 3);
            }
            _ => panic!("Expected Arena error with InvalidGeneration"),
        }

        // Test cross-arena access error
        let cross_error = ArenaErr::CrossArenaAccess { expected_id: 10, found_id: 20 };
        let tato_cross_error = TatoError::Arena(cross_error);

        match tato_cross_error {
            TatoError::Arena(ArenaErr::CrossArenaAccess { expected_id, found_id }) => {
                assert_eq!(expected_id, 10);
                assert_eq!(found_id, 20);
            }
            _ => panic!("Expected Arena error with CrossArenaAccess"),
        }
    }
}
