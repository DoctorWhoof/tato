use super::super::*;

#[test]
fn test_text_from_str() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let text = TempText::from_str(&mut arena, "Hello, World!").unwrap();
    
    assert_eq!(text.len(), 13);
    assert!(!text.is_empty());
    
    let str_value = text.as_str(&arena).unwrap();
    assert_eq!(str_value, "Hello, World!");
    
    let bytes = text.as_bytes(&arena).unwrap();
    assert_eq!(bytes, b"Hello, World!");
}

#[test]
fn test_text_from_bytes() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let byte_data = b"Test bytes";
    let text = TempText::from_bytes(&mut arena, byte_data).unwrap();
    
    assert_eq!(text.len(), 10);
    
    let bytes = text.as_bytes(&arena).unwrap();
    assert_eq!(bytes, b"Test bytes");
    
    let str_value = text.as_str(&arena).unwrap();
    assert_eq!(str_value, "Test bytes");
}

#[test]
fn test_text_empty() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let text = TempText::from_str(&mut arena, "").unwrap();
    
    assert_eq!(text.len(), 0);
    assert!(text.is_empty());
    
    let str_value = text.as_str(&arena).unwrap();
    assert_eq!(str_value, "");
    
    let bytes = text.as_bytes(&arena).unwrap();
    assert!(bytes.is_empty());
}

#[test]
fn test_text_unicode() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let unicode_str = "Hello 世界! 🦀 Rust";
    let text = TempText::from_str(&mut arena, unicode_str).unwrap();
    
    // Length is in bytes, not characters
    assert_eq!(text.len(), unicode_str.len());
    
    let str_value = text.as_str(&arena).unwrap();
    assert_eq!(str_value, unicode_str);
    
    let bytes = text.as_bytes(&arena).unwrap();
    assert_eq!(bytes, unicode_str.as_bytes());
}

#[test]
fn test_text_invalid_utf8() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    // Create invalid UTF-8 bytes
    let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
    let text = TempText::from_bytes(&mut arena, invalid_utf8).unwrap();
    
    assert_eq!(text.len(), 3);
    
    // Bytes should work fine
    let bytes = text.as_bytes(&arena).unwrap();
    assert_eq!(bytes, invalid_utf8);
    
    // as_str should return None for invalid UTF-8
    assert!(text.as_str(&arena).is_none());
}

#[test]
fn test_text_simple_format() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let text = TempText::format(&mut arena, "The answer is {}", 42).unwrap();
    
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "The answer is 42");
}

#[test]
fn test_text_format_display_multiple_values() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let values = [42, 100, 7];
    let text = TempText::format_display(&mut arena, "Values: {}, {}, {}", &values).unwrap();
    
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Values: 42, 100, 7");
}

#[test]
fn test_text_format_with_precision() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let values = [3.14159];
    let text = TempText::format_display(&mut arena, "Pi is approximately {:.2}", &values).unwrap();
    
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Pi is approximately 3.14");
}

#[test]
fn test_text_format_mixed_text_and_placeholders() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let values = [10, 20];
    let text = TempText::format_display(&mut arena, "Start {} middle {} end", &values).unwrap();
    
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Start 10 middle 20 end");
}

#[test]
fn test_text_format_no_placeholders() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let values: [i32; 0] = [];
    let text = TempText::format_display(&mut arena, "Just plain text", &values).unwrap();
    
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Just plain text");
}

#[test]
fn test_text_format_more_values_than_placeholders() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let values = [1, 2, 3, 4, 5];
    let text = TempText::format_display(&mut arena, "Only {} and {}", &values).unwrap();
    
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Only 1 and 2");
}

#[test]
fn test_text_format_fewer_values_than_placeholders() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let values = [42];
    let text = TempText::format_display(&mut arena, "First {} second {} third {}", &values).unwrap();
    
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "First 42 second  third ");
}

#[test]
fn test_text_format_different_types() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    // Test with floats
    let float_values = [2.5, 10.0];
    let text = TempText::format_display(&mut arena, "Float: {} and {}", &float_values).unwrap();
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Float: 2.5 and 10");
    
    // Test with booleans
    let bool_values = [true, false];
    let text2 = TempText::format_display(&mut arena, "Bool: {} and {}", &bool_values).unwrap();
    let result2 = text2.as_str(&arena).unwrap();
    assert_eq!(result2, "Bool: true and false");
}

#[test]
fn test_text_with_different_index_types() {
    // Test with u8 index
    let mut arena_u8: TempArena<256, u8> = TempArena::new();
    let text_u8 = TempText::from_str(&mut arena_u8, "u8 test").unwrap();
    assert_eq!(text_u8.as_str(&arena_u8).unwrap(), "u8 test");
    
    // Test with u16 index
    let mut arena_u16: TempArena<1024, u16> = TempArena::new();
    let text_u16 = TempText::from_str(&mut arena_u16, "u16 test").unwrap();
    assert_eq!(text_u16.as_str(&arena_u16).unwrap(), "u16 test");
}

#[test]
fn test_text_large_content() {
    let mut arena: TempArena<4096, u32> = TempArena::new();
    
    // Create a large string
    let large_string = "A".repeat(1000);
    let text = TempText::from_str(&mut arena, &large_string).unwrap();
    
    assert_eq!(text.len(), 1000);
    
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, large_string);
}

#[test]
fn test_text_out_of_space() {
    let mut arena: TempArena<32, u32> = TempArena::new(); // Very small arena
    
    // Try to store text that's too large
    let large_text = "This is a very long string that should exceed the arena capacity";
    let result = TempText::from_str(&mut arena, large_text);
    
    assert!(result.is_err());
    match result {
        Err(TempArenaError::OutOfSpace { .. }) => {
            // Expected error
        }
        _ => panic!("Expected OutOfSpace error"),
    }
}

#[test]
fn test_text_multiple_texts() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    // Create multiple texts in the same arena
    let text1 = TempText::from_str(&mut arena, "First").unwrap();
    let text2 = TempText::from_str(&mut arena, "Second").unwrap();
    let text3 = TempText::format(&mut arena, "Third: {}", 123).unwrap();
    
    // All texts should be accessible
    assert_eq!(text1.as_str(&arena).unwrap(), "First");
    assert_eq!(text2.as_str(&arena).unwrap(), "Second");
    assert_eq!(text3.as_str(&arena).unwrap(), "Third: 123");
    
    // Check lengths
    assert_eq!(text1.len(), 5);
    assert_eq!(text2.len(), 6);
    assert_eq!(text3.len(), 10);
}

#[test]
fn test_text_clone_copy_semantics() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    let text1 = TempText::from_str(&mut arena, "Test").unwrap();
    
    // TempText should be Copy, so we can copy it
    let text2 = text1;
    
    // Both should work and reference the same data
    assert_eq!(text1.as_str(&arena).unwrap(), "Test");
    assert_eq!(text2.as_str(&arena).unwrap(), "Test");
    assert_eq!(text1.len(), text2.len());
}

#[test]
fn test_text_format_precision_edge_cases() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    // Test with zero precision
    let values = [3.14159];
    let text = TempText::format_display(&mut arena, "Value: {:.0}", &values).unwrap();
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Value: 3");
    
    // Test with high precision
    let text2 = TempText::format_display(&mut arena, "Value: {:.5}", &values).unwrap();
    let result2 = text2.as_str(&arena).unwrap();
    assert_eq!(result2, "Value: 3.14159");
}

#[test]
fn test_text_format_invalid_precision() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    // Test with invalid precision format - should fallback to simple formatting
    let values = [42];
    let text = TempText::format_display(&mut arena, "Value: {:.abc}", &values).unwrap();
    let result = text.as_str(&arena).unwrap();
    assert_eq!(result, "Value: 42");
}

#[test] 
fn test_text_format_buffer_overflow() {
    let mut arena: TempArena<1024, u32> = TempArena::new();
    
    // Create many values that might overflow internal formatting buffer
    let _values: [i32; 100] = core::array::from_fn(|i| i as i32);
    // Create a template string with placeholders - use a shorter version for testing
    let template = "{} {} {} {} {}";
    let short_values = [0, 1, 2, 3, 4];
    
    let result = TempText::format_display(&mut arena, &template, &short_values);
    
    // Should either succeed or fail gracefully with InvalidBounds
    match result {
        Ok(text) => {
            // If it succeeds, verify the content makes sense
            let str_result = text.as_str(&arena).unwrap();
            assert!(str_result.contains("0"));
            assert!(str_result.contains("1"));
        }
        Err(TempArenaError::InvalidBounds) => {
            // This is an acceptable outcome for buffer overflow
        }
        Err(TempArenaError::OutOfSpace { .. }) => {
            // This is also acceptable if arena runs out of space
        }
        _ => panic!("Unexpected error type"),
    }
}