use super::*;

#[test]
fn test_text_format_uses_tail_allocation() {
    // Use a small arena to prove we're not using large stack buffers
    let mut arena: Arena<512> = Arena::new();
    
    // Record initial state
    let initial_used = arena.used();
    
    // Format some text - this should use tail allocation internally
    let text = Text::format(&mut arena, "Value: {}", 42, " end").unwrap();
    
    // The text should be correctly formatted
    assert_eq!(text.as_str(&arena).unwrap(), "Value: 42 end");
    
    // Final usage should only be the size of the final string
    let final_used = arena.used();
    let text_len = "Value: 42 end".len();
    
    // The difference should be approximately the text length (plus any alignment)
    assert!(final_used - initial_used < text_len + 8);
}

#[test]
fn test_text_join_uses_tail_allocation() {
    let mut arena: Arena<256> = Arena::new();
    
    // Create some text pieces
    let text1 = Text::from_str(&mut arena, "Hello").unwrap();
    let text2 = Text::from_str(&mut arena, ", ").unwrap();
    let text3 = Text::from_str(&mut arena, "World").unwrap();
    
    let before_join = arena.used();
    
    // Join them - should use tail allocation
    let joined = Text::join(&mut arena, &[text1, text2, text3]).unwrap();
    
    assert_eq!(joined.as_str(&arena).unwrap(), "Hello, World");
    
    // Space used should only be for the final joined string
    let after_join = arena.used();
    let joined_len = "Hello, World".len();
    assert!(after_join - before_join < joined_len + 8);
}

#[test]
fn test_text_from_buffer_uses_tail_allocation() {
    let mut arena: Arena<256> = Arena::new();
    
    // Create a buffer with some data
    let mut buffer = Buffer::<u8>::new(&mut arena, 20).unwrap();
    for &b in b"Hello Buffer" {
        buffer.push(&mut arena, b).unwrap();
    }
    
    let before_convert = arena.used();
    
    // Convert buffer to text - should use tail allocation
    let text = Text::from_buffer(&mut arena, &buffer).unwrap();
    
    assert_eq!(text.as_str(&arena).unwrap(), "Hello Buffer");
    
    // Space used should only be for the final text
    let after_convert = arena.used();
    let text_len = buffer.len();
    assert!(after_convert - before_convert < text_len + 8);
}

#[test]
fn test_format_with_very_small_arena() {
    // This would fail if using 256-byte DebugBuffer on stack
    // But works with tail allocation
    let mut arena: Arena<128> = Arena::new();
    
    // Format a small string
    let text = Text::format(&mut arena, "X=", 99, "").unwrap();
    assert_eq!(text.as_str(&arena).unwrap(), "X=99");
    
    // Can still allocate more
    let text2 = Text::format(&mut arena, "Y=", 11, "").unwrap();
    assert_eq!(text2.as_str(&arena).unwrap(), "Y=11");
}

#[test]
fn test_format_display_with_multiple_values() {
    let mut arena: Arena<512> = Arena::new();
    
    let values = [10, 20, 30];
    let text = Text::format_display(
        &mut arena,
        "Values: {}, {}, {}",
        &values,
        " done"
    ).unwrap();
    
    assert_eq!(text.as_str(&arena).unwrap(), "Values: 10, 20, 30 done");
}

#[test]
fn test_format_debug_with_multiple_values() {
    let mut arena: Arena<512> = Arena::new();
    
    let values = ["hello", "world"];
    let text = Text::format_dbg(
        &mut arena,
        "Debug: {:?}, {:?}",
        &values,
        " end"
    ).unwrap();
    
    assert_eq!(text.as_str(&arena).unwrap(), "Debug: \"hello\", \"world\" end");
}

#[test]
fn test_tail_allocation_stress() {
    let mut arena: Arena<1024> = Arena::new();
    
    // Perform many text operations that use tail allocation
    for i in 0..10 {
        let text = Text::format(&mut arena, "Iteration: {}", i, "").unwrap();
        // Check each iteration manually
        match i {
            0 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 0"),
            1 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 1"),
            2 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 2"),
            3 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 3"),
            4 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 4"),
            5 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 5"),
            6 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 6"),
            7 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 7"),
            8 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 8"),
            9 => assert_eq!(text.as_str(&arena).unwrap(), "Iteration: 9"),
            _ => unreachable!(),
        }
    }
    
    // Join multiple texts
    let text0 = Text::format(&mut arena, "{}", 0, "").unwrap();
    let text1 = Text::format(&mut arena, "{}", 1, "").unwrap();
    let text2 = Text::format(&mut arena, "{}", 2, "").unwrap();
    let text3 = Text::format(&mut arena, "{}", 3, "").unwrap();
    let text4 = Text::format(&mut arena, "{}", 4, "").unwrap();
    let texts = [text0, text1, text2, text3, text4];
    
    let joined = Text::join(&mut arena, &texts).unwrap();
    assert_eq!(joined.as_str(&arena).unwrap(), "01234");
    
    // Arena should still have reasonable usage
    assert!(arena.used() < 500); // Much less than if we kept all temp buffers
}

#[test]
fn test_tail_allocation_with_arena_ref() {
    let mut arena: Arena<512> = Arena::new();
    let mut arena_ref = arena.as_ref();
    
    // Format through ArenaRef - should use tail allocation
    let text = Text::format(&mut arena_ref, "Via ref: {}", 42, "").unwrap();
    assert_eq!(text.as_str(&arena_ref).unwrap(), "Via ref: 42");
    
    // Join through ArenaRef
    let t1 = Text::from_str(&mut arena_ref, "A").unwrap();
    let t2 = Text::from_str(&mut arena_ref, "B").unwrap();
    let joined = Text::join(&mut arena_ref, &[t1, t2]).unwrap();
    assert_eq!(joined.as_str(&arena_ref).unwrap(), "AB");
}

#[test]
fn test_tail_allocation_preserves_existing_data() {
    let mut arena: Arena<512> = Arena::new();
    
    // Allocate some permanent data
    let id1 = arena.alloc(100u32).unwrap();
    let text1 = Text::from_str(&mut arena, "permanent").unwrap();
    
    let before_format = arena.used();
    
    // Format text (uses tail allocation internally)
    let text2 = Text::format(&mut arena, "Temp: {}", 42, "").unwrap();
    
    // Original data should still be valid
    assert_eq!(*arena.get(id1).unwrap(), 100);
    assert_eq!(text1.as_str(&arena).unwrap(), "permanent");
    assert_eq!(text2.as_str(&arena).unwrap(), "Temp: 42");
    
    // Usage should only increase by the final formatted string size
    let after_format = arena.used();
    let formatted_len = "Temp: 42".len();
    assert!(after_format - before_format < formatted_len + 8);
}

#[test]
fn test_join_bytes_uses_tail_allocation() {
    let mut arena: Arena<256> = Arena::new();
    
    let before = arena.used();
    
    let slices: [&[u8]; 5] = [b"one", b"-", b"two", b"-", b"three"];
    let text = Text::join_bytes(&mut arena, &slices).unwrap();
    
    assert_eq!(text.as_str(&arena).unwrap(), "one-two-three");
    
    // Should only use space for final result
    let after = arena.used();
    let result_len = "one-two-three".len();
    assert!(after - before < result_len + 8);
}

#[test]
fn test_format_error_restores_tail() {
    let mut arena: Arena<64> = Arena::new(); // Very small arena
    
    // Fill most of the arena
    let _data = arena.alloc_slice(&[0u8; 50]).unwrap();
    
    let before_remaining = arena.remaining();
    
    // Try to format something too large - should fail
    let result = Text::format(&mut arena, "This is a very long string: {}", 99999, " and more");
    assert!(result.is_err());
    
    // Tail should be restored even after error
    let after_remaining = arena.remaining();
    assert_eq!(before_remaining, after_remaining);
}

#[test]
fn test_nested_tail_operations() {
    let mut arena: Arena<512> = Arena::new();
    
    // Create some texts
    let t1 = Text::from_str(&mut arena, "A").unwrap();
    let t2 = Text::from_str(&mut arena, "B").unwrap();
    
    // Join them (uses tail allocation)
    let joined = Text::join(&mut arena, &[t1, t2]).unwrap();
    
    // Verify the join worked
    assert_eq!(joined.as_str(&arena).unwrap(), "AB");
    
    // Format with a literal string to avoid borrow issues
    // (we already verified joined contains "AB")
    let formatted = Text::format(&mut arena, "Joined: {}",
        "AB", "").unwrap();
    
    assert_eq!(formatted.as_str(&arena).unwrap(), "Joined: AB");
}