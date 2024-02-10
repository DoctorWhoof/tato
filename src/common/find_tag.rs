
pub const fn find_tag(name:&str, bytes:&[u8]) -> u8  {
    let name = name.as_bytes();
    let mut index = 0;
    let mut name_index = 0;
    let mut current_match = name[0];
    while index < bytes.len() {
        if bytes[index] == current_match {
            name_index += 1;
            if name_index == name.len() {
                return bytes[index + 1];
            }
            current_match = name[name_index];
        } else {
            name_index = 0;
            current_match = name[0];
        };
        index += 1
    }
    panic!("Tag not found!")
}