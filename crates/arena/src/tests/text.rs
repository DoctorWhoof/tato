use super::*;

#[test]
fn test_text_from_str() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::from_str(&mut arena, "Hello, World!").unwrap();
    assert_eq!(text.len(), 13);

    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Hello, World!");
}

#[test]
fn test_text_format_display_single() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "value: {}", 42, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "value: 42");
}

#[test]
fn test_text_format_debug_single() {
    let mut arena: Arena<1024> = Arena::new();

    let text = Text::format(&mut arena, "debug: {:?}", "Hello", "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "debug: \"Hello\"");
}

#[test]
fn test_text_format_display_indexed() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [42, 100];
    let text = Text::format_display(&mut arena, "first: {}, second: {}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "first: 42, second: 100");
}

#[test]
fn test_text_format_debug_indexed() {
    let mut arena: Arena<1024> = Arena::new();

    let values = ["hello", "world"];
    let text = Text::format_dbg(&mut arena, "first: {:?}, second: {:?}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "first: \"hello\", second: \"world\"");
}

#[test]
fn test_text_format_display_precision() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [3.14159, 2.71828];
    let text = Text::format_display(&mut arena, "pi: {:.2}, e: {:.3}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "pi: 3.14, e: 2.718");
}



#[test]
fn test_text_format_empty_values() {
    let mut arena: Arena<1024> = Arena::new();

    let values: &[i32] = &[];
    let text = Text::format_display(&mut arena, "no placeholders", values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "no placeholders");
}

#[test]
fn test_text_format_debug_with_display_specifier() {
    let mut arena: Arena<1024> = Arena::new();

    // format_dbg should now accept {} specifiers but still use Debug formatting
    let values = ["hello", "world"];
    let text = Text::format_dbg(&mut arena, "first: {}, second: {}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "first: \"hello\", second: \"world\"");

    // Test with precision specifier too
    let values = [42];
    let text = Text::format_dbg(&mut arena, "value: {:.2}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "value: 42"); // Debug formatting ignores precision for integers
}

#[test]
#[should_panic(expected = "precision with debug (?), use either {:.N} or {:?}")]
fn test_text_format_invalid_precision_debug() {
    let mut arena: Arena<1024> = Arena::new();

    // This should panic with a clear error message
    let values = [42];
    let _text = Text::format_dbg(&mut arena, "value: {:.1?}", &values, "");
}

#[test]
#[should_panic(expected = "Invalid format string: found '{' without matching '}'")]
fn test_text_format_unmatched_brace_open() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [42];
    let _text = Text::format_display(&mut arena, "value: {", &values, "");
}

#[test]
#[should_panic(expected = "Invalid format string: found '}' without matching '{'")]
fn test_text_format_unmatched_brace_close() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [42];
    let _text = Text::format_display(&mut arena, "value: }", &values, "");
}

#[test]
#[should_panic(expected = "Invalid format specifier: supported formats are {}, {:?}, {:.N}")]
fn test_text_format_invalid_specifier() {
    let mut arena: Arena<1024> = Arena::new();

    let values = [42];
    let _text = Text::format_display(&mut arena, "value: {:x}", &values, "");
}

#[test]
fn test_text_format_with_second_message() {
    let mut arena: Arena<1024> = Arena::new();

    // Test format_display with second message
    let values = [42, 100];
    let text = Text::format_display(&mut arena, "Values: {}, {}", &values, " (end)").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Values: 42, 100 (end)");

    // Test format_dbg with second message
    let values = ["hello"];
    let text = Text::format_dbg(&mut arena, "Debug: {}", &values, " done").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Debug: \"hello\" done");

    // Test with empty second message (should work like before)
    let values = [3.14];
    let text = Text::format_display(&mut arena, "Pi: {:.2}", &values, "").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Pi: 3.14");
}

#[test]
fn test_text_format_practical_two_message_examples() {
    let mut arena: Arena<1024> = Arena::new();

    // Example: "Temperature: 23 degrees"
    let temp_values = [23];
    let text = Text::format_display(&mut arena, "Temperature: {}", &temp_values, " degrees").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Temperature: 23 degrees");

    // Example: "Loading... 45% complete"
    let progress_values = [45];
    let text = Text::format_display(&mut arena, "Loading... {}%", &progress_values, " complete").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Loading... 45% complete");

    // Example with debug: "Player position: (10, 20) in world space"
    let pos_values = [(10, 20)];
    let text = Text::format_dbg(&mut arena, "Player position: {}", &pos_values, " in world space").unwrap();
    let s = text.as_str(&arena).unwrap();
    assert_eq!(s, "Player position: (10, 20) in world space");
}

#[test]
fn test_text_from_buffer_basic() {
    let mut arena: Arena<1024> = Arena::new();

    // Create a buffer with some text data
    let mut buffer = Buffer::new(&mut arena, 20u32).unwrap();
    let test_str = "Hello, world!";

    // Fill buffer with test data
    for &byte in test_str.as_bytes() {
        buffer.push(&mut arena, byte).unwrap();
    }

    // Convert buffer to text using tail allocation
    let text = Text::from_buffer(&mut arena, &buffer).unwrap();

    // Verify the text matches
    let text_str = text.as_str(&arena).unwrap();
    assert_eq!(text_str, test_str);
    assert_eq!(text.len(), test_str.len());
}

#[test]
fn test_text_from_buffer_empty() {
    let mut arena: Arena<1024> = Arena::new();

    // Create an empty buffer
    let buffer = Buffer::new(&mut arena, 10u32).unwrap();

    // Convert empty buffer to text
    let text = Text::from_buffer(&mut arena, &buffer).unwrap();

    // Should be empty
    assert_eq!(text.len(), 0);
    assert!(text.is_empty());
    let text_str = text.as_str(&arena).unwrap();
    assert_eq!(text_str, "");
}

#[test]
fn test_text_from_buffer_partial_usage() {
    let mut arena: Arena<1024> = Arena::new();

    // Create a buffer with large capacity but only use part of it
    let mut buffer = Buffer::new(&mut arena, 100u32).unwrap();
    let test_str = "Hi!";

    // Only fill first few bytes
    for &byte in test_str.as_bytes() {
        buffer.push(&mut arena, byte).unwrap();
    }

    // Convert buffer to text - should only copy the used portion
    let text = Text::from_buffer(&mut arena, &buffer).unwrap();

    let text_str = text.as_str(&arena).unwrap();
    assert_eq!(text_str, test_str);
    assert_eq!(text.len(), 3); // Only the used portion
}

#[test]
fn test_text_from_buffer_sequential_operations() {
    let mut arena: Arena<512> = Arena::new();

    // First, create some regular allocations to use up front space
    let _some_data = arena.alloc_slice_from_fn(100u32, |i| i as u8).unwrap();

    // Create a buffer
    let mut buffer = Buffer::new(&mut arena, 50u32).unwrap();
    let test_str = "Test string";

    for &byte in test_str.as_bytes() {
        buffer.push(&mut arena, byte).unwrap();
    }

    // Convert buffer to text - this should work even with existing allocations
    let text = Text::from_buffer(&mut arena, &buffer).unwrap();

    let text_str = text.as_str(&arena).unwrap();
    assert_eq!(text_str, test_str);

    // Should be able to do more allocations afterward
    let _more_data = arena.alloc_slice_from_fn(20u32, |i| (i + 42) as u8).unwrap();
}

#[test]
fn test_text_from_buffer_near_capacity() {
    let mut arena: Arena<256> = Arena::new();

    // Fill up most of the arena first
    let _filler = arena.alloc_slice_from_fn(200u32, |i| i as u8).unwrap();

    // Create a small buffer
    let mut buffer = Buffer::new(&mut arena, 10u32).unwrap();
    let test_str = "Small";

    for &byte in test_str.as_bytes() {
        buffer.push(&mut arena, byte).unwrap();
    }

    // This should still work with tail allocation
    let text = Text::from_buffer(&mut arena, &buffer).unwrap();

    let text_str = text.as_str(&arena).unwrap();
    assert_eq!(text_str, test_str);
}

#[test]
fn test_text_from_buffer_out_of_space() {
    let mut arena: Arena<32> = Arena::new(); // Very small arena

    // Fill up most of the arena
    let _filler = arena.alloc_slice_from_fn(20u32, |i| i as u8).unwrap();

    // Create a buffer with the remaining space
    let mut buffer = Buffer::new(&mut arena, 8u32).unwrap();
    let test_str = "Hello"; // 5 bytes

    for &byte in test_str.as_bytes() {
        buffer.push(&mut arena, byte).unwrap();
    }

    // Fill remaining space to leave insufficient room for tail allocation
    // We need 5 bytes for tail + 5 bytes for final = 10 bytes total
    // But leave only ~4 bytes available
    let remaining = arena.remaining();
    if remaining > 2 {
        let fill_size = u32::try_from(remaining - 2).unwrap();
        let _more_filler = arena.alloc_slice_from_fn(fill_size, |i| (i + 200) as u8).unwrap();
    }

    // Try to convert - should fail due to insufficient space for tail + final allocation
    let result = Text::from_buffer(&mut arena, &buffer);
    assert!(result.is_err());

    match result {
        Err(ArenaError::OutOfSpace { requested: _, available: _ }) => {
            // This is expected
        }
        _ => panic!("Expected OutOfSpace error"),
    }
}

#[test]
fn test_text_from_buffer_utf8_validation() {
    let mut arena: Arena<1024> = Arena::new();

    // Create buffer with valid UTF-8 bytes
    let mut buffer = Buffer::new(&mut arena, 20u32).unwrap();
    let utf8_str = "Hello 世界! 🚀";

    for &byte in utf8_str.as_bytes() {
        buffer.push(&mut arena, byte).unwrap();
    }

    let text = Text::from_buffer(&mut arena, &buffer).unwrap();

    let text_str = text.as_str(&arena).unwrap();
    assert_eq!(text_str, utf8_str);
}

#[test]
fn test_text_from_buffer_invalid_utf8() {
    let mut arena: Arena<1024> = Arena::new();

    // Create buffer with invalid UTF-8 sequence
    let mut buffer = Buffer::new(&mut arena, 10u32).unwrap();

    // Add some valid bytes, then invalid UTF-8
    buffer.push(&mut arena, b'H').unwrap();
    buffer.push(&mut arena, b'i').unwrap();
    buffer.push(&mut arena, 0xFF).unwrap(); // Invalid UTF-8
    buffer.push(&mut arena, 0xFE).unwrap(); // Invalid UTF-8

    let text = Text::from_buffer(&mut arena, &buffer).unwrap();

    // The text should be created (copying is successful)
    // but as_str should return None due to invalid UTF-8
    assert_eq!(text.len(), 4);
    assert!(text.as_str(&arena).is_none());
}

#[test]
fn test_text_from_buffer_multiple_conversions() {
    let mut arena: Arena<1024> = Arena::new();

    // Create multiple buffers and convert them sequentially
    let test_strings = ["First", "Second", "Third"];
    let mut texts = [Text::default(), Text::default(), Text::default()];

    for (i, test_str) in test_strings.iter().enumerate() {
        let mut buffer = Buffer::new(&mut arena, 20u32).unwrap();

        for &byte in test_str.as_bytes() {
            buffer.push(&mut arena, byte).unwrap();
        }

        let text = Text::from_buffer(&mut arena, &buffer).unwrap();
        texts[i] = text;
    }

    // Verify all texts
    for (i, text) in texts.iter().enumerate() {
        let text_str = text.as_str(&arena).unwrap();
        assert_eq!(text_str, test_strings[i]);
    }
}

#[test]
fn test_text_join_slices_basic() {
    let mut arena: Arena<1024> = Arena::new();

    let slice1 = b"Hello, ";
    let slice2 = b"world!";
    let slices = [slice1.as_slice(), slice2.as_slice()];

    let text = Text::join_bytes(&mut arena, &slices).unwrap();
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Hello, world!");
    assert_eq!(text.len(), 13);
}

#[test]
fn test_text_join_slices_empty() {
    let mut arena: Arena<1024> = Arena::new();

    let slices: [&[u8]; 0] = [];
    let text = Text::join_bytes(&mut arena, &slices).unwrap();
    assert_eq!(text.len(), 0);

    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_text_join_slices_single() {
    let mut arena: Arena<1024> = Arena::new();

    let slice = b"Single slice";
    let slices = [slice.as_slice()];

    let text = Text::join_bytes(&mut arena, &slices).unwrap();
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Single slice");
    assert_eq!(text.len(), 12);
}

#[test]
fn test_text_join_slices_multiple() {
    let mut arena: Arena<1024> = Arena::new();

    let slice1 = b"One";
    let slice2 = b" ";
    let slice3 = b"Two";
    let slice4 = b" ";
    let slice5 = b"Three";
    let slices = [
        slice1.as_slice(),
        slice2.as_slice(),
        slice3.as_slice(),
        slice4.as_slice(),
        slice5.as_slice(),
    ];

    let text = Text::join_bytes(&mut arena, &slices).unwrap();
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "One Two Three");
    assert_eq!(text.len(), 13);
}

#[test]
fn test_text_join_slices_with_empty_slices() {
    let mut arena: Arena<1024> = Arena::new();

    let slice1 = b"Start";
    let slice2 = b"";
    let slice3 = b"Middle";
    let slice4 = b"";
    let slice5 = b"End";
    let slices = [
        slice1.as_slice(),
        slice2.as_slice(),
        slice3.as_slice(),
        slice4.as_slice(),
        slice5.as_slice(),
    ];

    let text = Text::join_bytes(&mut arena, &slices).unwrap();
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "StartMiddleEnd");
    assert_eq!(text.len(), 14);
}

#[test]
fn test_text_join_slices_large() {
    let mut arena: Arena<1024> = Arena::new();

    // Create several slices with different content
    let slice1 = b"The quick brown fox ";
    let slice2 = b"jumps over the lazy ";
    let slice3 = b"dog in the park.";
    let slices = [slice1.as_slice(), slice2.as_slice(), slice3.as_slice()];

    let text = Text::join_bytes(&mut arena, &slices).unwrap();
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "The quick brown fox jumps over the lazy dog in the park.");
    assert_eq!(text.len(), 56);
}
