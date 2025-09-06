use super::*;
use core::{array::from_fn, ops::Range, str::from_utf8};

// Arena allocated parsed command
#[derive(Debug, Clone)]
pub struct Command {
    pub data: [u8; COMMAND_MAX_LEN as usize],
    item_ranges: [Range<u8>; COMMAND_MAX_ARGS],
    item_count: u8,
}

impl Command {
    pub fn parse_str(text: &str) -> Option<Command> {
        let text_slice = text.as_bytes();
        let bytes: [u8; COMMAND_MAX_LEN as usize] =
            from_fn(|i| if i < text.len() { text_slice[i] } else { 0 });

        let mut item_ranges: [Range<u8>; COMMAND_MAX_ARGS] = from_fn(|_| Range::default());
        let mut seg_start = 0;
        let mut item_count = 0;
        let mut is_space = true;

        for i in 0..text.len() as u8 {
            let ch = bytes[i as usize];
            if ch == ' ' as u8 || ch == ',' as u8 || ch == '(' as u8 || ch == ')' as u8 {
                // Validate
                if is_space {
                    // Will ignore whitespaces
                    seg_start += 1;
                    continue;
                }
                // Passed, this marks a new item
                item_ranges[item_count as usize] = seg_start..i;
                seg_start = i + 1;
                item_count += 1;
                is_space = true;
            } else {
                // Not a space, we're in the middle of an item
                is_space = false;
            }
        }

        // Handle the final segment after the last space
        if seg_start < text.len() as u8 && !is_space {
            item_ranges[item_count as usize] = seg_start..text.len() as u8;
            item_count += 1;
        }

        if item_count == 0 {
            return None;
        };

        let bytes = text.as_bytes();
        Some(Command {
            data: from_fn(|i| if i < text.len() { bytes[i] } else { 0 }),
            item_ranges,
            item_count,
        })
    }

    pub fn item_count(&self) -> u8 {
        self.item_count
    }

    pub fn name<'a>(&'a self) -> &'a str {
        let range = &self.item_ranges[0];
        from_utf8(&self.data[range.start as usize..range.end as usize]).unwrap()
    }

    pub fn arg<'a>(&'a self, index: u8) -> Option<&'a str> {
        if index >= self.item_count - 1 {
            return None;
        }
        let range = &self.item_ranges[index as usize + 1];
        from_utf8(&self.data[range.start as usize..range.end as usize]).ok()
    }

    #[inline]
    fn range(&self, index: u8) -> Range<u8> {
        self.item_ranges[index as usize].clone()
    }
}

#[test]
fn test_commmand_parse() {
    let empty = Command::parse_str("");
    assert!(empty.is_none());

    let simple = Command::parse_str("SimpleCommand").unwrap();
    assert_eq!(simple.item_count(), 1);
    assert_eq!(simple.name(), "SimpleCommand");
    assert_eq!(simple.arg(0), None);

    let preceding_space = Command::parse_str(" Test").unwrap();
    assert_eq!(preceding_space.item_count(), 1);
    assert_eq!(preceding_space.name(), "Test");
    assert_eq!(preceding_space.arg(0), None);

    let whitespaced = Command::parse_str(" Test   arg0    arg1   ").unwrap();
    assert_eq!(whitespaced.item_count(), 3);
    assert_eq!(whitespaced.name(), "Test");
    assert_eq!(whitespaced.arg(0), Some("arg0"));
    assert_eq!(whitespaced.arg(1), Some("arg1"));
    assert_eq!(whitespaced.arg(2), None);

    let command = Command::parse_str("CommandName arg0 arg1 arg2").unwrap();
    assert_eq!(command.item_count(), 4);
    assert_eq!(command.name(), "CommandName");
    assert_eq!(command.arg(0), Some("arg0"));
    assert_eq!(command.arg(1), Some("arg1"));
    assert_eq!(command.arg(2), Some("arg2"));
    assert_eq!(command.arg(3), None);
}
