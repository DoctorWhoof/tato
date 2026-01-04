// use super::*;

// #[test]
// fn test_tail_alloc_basic() {
//     let mut arena: Arena<1024> = Arena::new();

//     // Create some data in a slice
//     let source_slice = arena.alloc_slice_from_fn(10, |i| (i + 65) as u8).unwrap(); // "ABCDEFGHIJ"

//     // Copy the data to a local buffer first to avoid borrow issues
//     let mut temp_data = [0u8; 10];
//     temp_data.copy_from_slice(arena.get_slice(source_slice).unwrap());

//     // Use tail allocation to copy it
//     // let copied_slice = arena.copy_slice_via_tail(&temp_data).unwrap();

//     // Verify the copy is correct
//     let source_data = arena.get_slice(source_slice).unwrap();
//     let copied_data = arena.get_slice(copied_slice).unwrap();

//     assert_eq!(source_data, copied_data);
//     assert_eq!(copied_data.len(), 10);
//     assert_eq!(copied_data[0], 65); // 'A'
//     assert_eq!(copied_data[9], 74); // 'J'
// }

// #[test]
// fn test_tail_alloc_partial_copy() {
//     let mut arena: Arena<1024> = Arena::new();

//     // Create a longer slice but only copy part of it
//     let source_slice = arena.alloc_slice_from_fn(20, |i| (i + 48) as u8).unwrap(); // "0123456789..."

//     // Copy first 5 bytes to a local buffer
//     let mut partial_data = [0u8; 5];
//     partial_data.copy_from_slice(&arena.get_slice(source_slice).unwrap()[..5]);

//     // Only copy first 5 bytes
//     let copied_slice = arena.copy_slice_via_tail(&partial_data).unwrap();

//     let copied_data = arena.get_slice(copied_slice).unwrap();

//     assert_eq!(copied_data.len(), 5);
//     assert_eq!(copied_data[0], 48); // '0'
//     assert_eq!(copied_data[4], 52); // '4'
// }

// #[test]
// fn test_tail_alloc_with_existing_allocations() {
//     let mut arena: Arena<512> = Arena::new();

//     // Make some existing allocations
//     let _existing1 = arena.alloc_slice_from_fn(100, |i| i as u8).unwrap();
//     let _existing2 = arena.alloc_slice_from_fn(50, |i| (i + 100) as u8).unwrap();

//     // Create source data
//     let source_slice = arena.alloc_slice_from_fn(30, |i| (i + 200) as u8).unwrap();

//     // Copy to a local buffer
//     let mut temp_data = [0u8; 30];
//     temp_data.copy_from_slice(arena.get_slice(source_slice).unwrap());

//     // Use tail allocation - should work with existing allocations
//     let copied_slice = arena.copy_slice_via_tail(&temp_data).unwrap();

//     let source_data = arena.get_slice(source_slice).unwrap();
//     let copied_data = arena.get_slice(copied_slice).unwrap();

//     assert_eq!(source_data, copied_data);
//     assert_eq!(copied_data.len(), 30);
// }

// #[test]
// fn test_tail_alloc_insufficient_space() {
//     let mut arena: Arena<64> = Arena::new(); // Very small arena

//     // Fill up most of the arena
//     let _filler = arena.alloc_slice_from_fn(50, |i| i as u8).unwrap();

//     // Create a small source slice
//     let source_slice = arena.alloc_slice_from_fn(8, |i| (i + 100) as u8).unwrap();

//     // Copy to a local buffer
//     let mut temp_data = [0u8; 8];
//     temp_data.copy_from_slice(arena.get_slice(source_slice).unwrap());

//     // Try to copy - should fail because we need space for both tail copy AND final allocation
//     // With only ~6 bytes left, we can't fit 8 bytes for tail + 8 bytes for final
//     let result = arena.copy_slice_via_tail(&temp_data);

//     assert!(result.is_err());
//     match result {
//         Err(ArenaErr::OutOfSpace { requested: _, available: _ }) => {
//             // Expected
//         }
//         _ => panic!("Expected OutOfSpace error"),
//     }
// }

// #[test]
// fn test_tail_alloc_exactly_fits() {
//     let mut arena: Arena<128> = Arena::new();

//     // Calculate space carefully - need room for source + tail copy + final copy
//     let source_size = 20usize;
//     let source_slice = arena.alloc_slice_from_fn(source_size, |i| (i + 65) as u8).unwrap();

//     // Copy source data to local buffer
//     let mut source_data = [0u8; 20];
//     source_data.copy_from_slice(arena.get_slice(source_slice).unwrap());

//     // Fill arena to leave exactly enough space for tail allocation
//     let _used = arena.used();
//     let remaining = arena.remaining();

//     // We need: tail_space + final_space >= 2 * source_size
//     // Leave exactly enough space
//     let fill_amount = remaining.saturating_sub(2 * source_size);
//     if fill_amount > 0 {
//         let _filler = arena.alloc_slice_from_fn(fill_amount, |i| i as u8).unwrap();
//     }

//     // Now try the tail allocation - should just barely fit
//     let copied_slice = arena.copy_slice_via_tail(&source_data).unwrap();

//     let copied_data = arena.get_slice(copied_slice).unwrap();
//     assert_eq!(copied_data.len(), 20);
// }

// #[test]
// fn test_tail_alloc_zero_length() {
//     let mut arena: Arena<1024> = Arena::new();

//     // Create a slice but copy zero length
//     let _source_slice = arena.alloc_slice_from_fn(10, |i| (i + 100) as u8).unwrap();

//     let empty_data: &[u8] = &[];
//     let copied_slice = arena.copy_slice_via_tail(empty_data).unwrap();

//     let copied_data = arena.get_slice(copied_slice).unwrap();
//     assert_eq!(copied_data.len(), 0);
// }

// #[test]
// fn test_tail_alloc_bytes_basic() {
//     let mut arena: Arena<1024> = Arena::new();

//     // Test tail allocation of raw bytes
//     let data = [1u8, 2, 3, 4, 5];
//     let ptr = arena.tail_alloc_bytes(data.len(), 1).unwrap();

//     unsafe {
//         // Write data to the tail allocation
//         core::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());

//         // Read it back
//         let read_back = core::slice::from_raw_parts(ptr, data.len());
//         assert_eq!(read_back, &data[..]);
//     }
// }

// #[test]
// fn test_tail_alloc_bytes_alignment() {
//     let mut arena: Arena<1024> = Arena::new();

//     // Test alignment requirements for u64
//     let align = core::mem::align_of::<u64>();
//     let size = core::mem::size_of::<u64>();

//     let ptr = arena.tail_alloc_bytes(size, align).unwrap();
//     assert_eq!(ptr as usize % align, 0, "Pointer should be aligned to {}", align);

//     unsafe {
//         // Should be able to write and read u64 at this aligned address
//         let u64_ptr = ptr as *mut u64;
//         *u64_ptr = 0xDEADBEEF_CAFEBABE;
//         assert_eq!(*u64_ptr, 0xDEADBEEF_CAFEBABE);
//     }
// }

// #[test]
// fn test_tail_alloc_bytes_insufficient_space() {
//     let mut arena: Arena<32> = Arena::new(); // Very small arena

//     // Fill up the arena
//     let _filler = arena.alloc_slice_from_fn(28, |i| i as u8).unwrap();

//     // Try to allocate more than remaining
//     let result = arena.tail_alloc_bytes(10, 1);
//     assert!(result.is_err());
// }

// #[test]
// fn test_tail_alloc_mixed_types() {
//     let mut arena: Arena<1024> = Arena::new();

//     // Copy different types via tail allocation
//     let u32_data = [1u32, 2, 3, 4];
//     let u32_copy = arena.copy_slice_via_tail(&u32_data).unwrap();

//     let f64_data = [1.1f64, 2.2, 3.3];
//     let f64_copy = arena.copy_slice_via_tail(&f64_data).unwrap();

//     // Verify copies
//     assert_eq!(arena.get_slice(u32_copy).unwrap(), &[1u32, 2, 3, 4]);
//     assert_eq!(arena.get_slice(f64_copy).unwrap(), &[1.1f64, 2.2, 3.3]);
// }

// #[test]
// fn test_tail_alloc_stress() {
//     let mut arena: Arena<4096> = Arena::new();

//     // Perform multiple tail allocations in sequence
//     for i in 0..10 {
//         let size = 10 + i * 5;

//         // Create source data directly
//         let mut source_data = [0u8; 60]; // Max size is 10 + 9*5 = 55
//         for j in 0..size {
//             source_data[j] = (j + i * 10) as u8;
//         }
//         let source_data = &source_data[..size];

//         let copied = arena.copy_slice_via_tail(&source_data).unwrap();

//         // Verify the copy
//         let copied_data = arena.get_slice(copied).unwrap();
//         assert_eq!(copied_data.len(), size);
//         for j in 0..size {
//             assert_eq!(copied_data[j], (j + i * 10) as u8);
//         }
//     }
// }

// #[test]
// fn test_tail_alloc_with_clear() {
//     let mut arena: Arena<512> = Arena::new();

//     // Allocate and copy
//     let source_data = [1u8, 2, 3, 4, 5];
//     let source = arena.alloc_slice(&source_data).unwrap();
//     let copied = arena.copy_slice_via_tail(&source_data).unwrap();

//     let gen_before = arena.generation();

//     // Clear arena
//     arena.clear();

//     // Generation should have changed
//     assert_eq!(arena.generation(), gen_before + 1);

//     // Old slices should be invalid
//     assert!(arena.get_slice(source).is_err());
//     assert!(arena.get_slice(copied).is_err());

//     // Can allocate new data
//     let new_slice = arena.alloc_slice(&[10, 20, 30]).unwrap();
//     assert_eq!(arena.get_slice(new_slice).unwrap(), &[10, 20, 30]);
// }
