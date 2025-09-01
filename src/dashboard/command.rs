use super::*;
use core::{array::from_fn, ops::Range, str::from_utf8};

// Arena allocated parsed command
#[derive(Debug, Clone)]
pub struct Command {
    data: Text,
    item_ranges: Slice<Range<u8>>,
    item_count: u8,
}

impl Command {
    pub fn parse_str<const LEN: usize>(text: &str, arena: &mut Arena<LEN>) -> Option<Command> {
        let text_slice = text.as_bytes();
        let bytes: [u8; COMMAND_MAX_LEN as usize] =
            from_fn(|i| if i < text.len() { text_slice[i] } else { 0 });

        let mut temp_ranges: [Range<u8>; COMMAND_MAX_ARGS] = from_fn(|_| Range::default());
        let mut seg_start = 0;
        let mut item_count = 0;
        let mut is_space = true;

        for i in 0..text.len() as u8 {
            if bytes[i as usize] == ' ' as u8 {
                // Validate
                if is_space {
                    // Will ignore whitespaces
                    seg_start += 1;
                    continue;
                }
                // Passed, this marks a new item
                temp_ranges[item_count as usize] = seg_start..i;
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
            temp_ranges[item_count as usize] = seg_start..text.len() as u8;
            item_count += 1;
        }

        if item_count == 0 {
            return None;
        };

        Some(Command {
            data: Text::from_str(arena, text).unwrap(),
            item_ranges: arena
                .alloc_slice_from_fn(item_count as u32, |i| {
                    if i < item_count as usize {
                        temp_ranges[i].clone() // Why is Range not "Copy"? Nobody knows...
                    } else {
                        unreachable!()
                    }
                })
                .unwrap(),
            item_count,
        })
    }

    pub fn item_count(&self) -> u8 {
        self.item_count
    }

    pub fn name<'a, const LEN: usize>(&self, arena: &'a Arena<LEN>) -> &'a str {
        let range = &self.range(0, arena);
        let slice = self.data.as_slice(arena).unwrap();
        from_utf8(&slice[range.start as usize..range.end as usize]).unwrap()
    }

    pub fn arg<'a, const LEN: usize>(&self, index: u8, arena: &'a Arena<LEN>) -> Option<&'a str> {
        if index >= self.item_count - 1 {
            return None;
        }
        let slice = self.data.as_slice(arena).unwrap();
        let range = self.range(index + 1, arena);
        from_utf8(&slice[range.start as usize..range.end as usize]).ok()
    }

    #[inline]
    fn range<const LEN: usize>(&self, index: u8, arena: &Arena<LEN>) -> Range<u8> {
        let ranges = arena.get_slice(&self.item_ranges).unwrap();
        ranges[index as usize].clone()
    }
}

#[test]
fn test_commmand_parse() {
    let mut arena = Arena::<1024, u32>::new();

    let empty = Command::parse_str("", &mut arena);
    assert!(empty.is_none());

    let simple = Command::parse_str("SimpleCommand", &mut arena).unwrap();
    assert_eq!(simple.item_count(), 1);
    assert_eq!(simple.name(&arena), "SimpleCommand");
    assert_eq!(simple.arg(0, &arena), None);

    let preceding_space = Command::parse_str(" Test", &mut arena).unwrap();
    assert_eq!(preceding_space.item_count(), 1);
    assert_eq!(preceding_space.name(&arena), "Test");
    assert_eq!(preceding_space.arg(0, &arena), None);

    let whitespaced = Command::parse_str(" Test   arg0    arg1   ", &mut arena).unwrap();
    assert_eq!(whitespaced.item_count(), 3);
    assert_eq!(whitespaced.name(&arena), "Test");
    assert_eq!(whitespaced.arg(0, &arena), Some("arg0"));
    assert_eq!(whitespaced.arg(1, &arena), Some("arg1"));
    assert_eq!(whitespaced.arg(2, &arena), None);

    let command = Command::parse_str("CommandName arg0 arg1 arg2", &mut arena).unwrap();
    assert_eq!(command.item_count(), 4);
    assert_eq!(command.name(&arena), "CommandName");
    assert_eq!(command.arg(0, &arena), Some("arg0"));
    assert_eq!(command.arg(1, &arena), Some("arg1"));
    assert_eq!(command.arg(2, &arena), Some("arg2"));
    assert_eq!(command.arg(3, &arena), None);
}
