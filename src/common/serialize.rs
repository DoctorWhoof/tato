#[doc(hidden)]
/// Deserialization helper
pub struct Cursor<'a, T:Copy> {
    data:&'a[T],
    head:usize
}

/// Like a minimal iterator, but doesn't return option (simply crashes if you go too far).
impl<'a, T:Copy> Cursor<'a,T> {
    // TODO: Add more methods like "next_u16" that joins up the data into bytes internally
    pub fn new(data:&'a[T]) -> Self {
        Self {
            data,
            head:0
        }
    }

    pub fn advance(&mut self) -> T {
        let result = self.data[self.head];
        self.head += 1;
        result
    }
}

/// Helps building up a byte array for serialization. Similar to Pool.
pub struct ByteArray<const LEN:usize> {
    // TODO: Maybe just use an improved Pool<u8>?
    // TODO: Add more methods like "push_u16" that breaks up the data into bytes internally
    pub data:[u8; LEN],
    head: usize,
    tail: usize
}

impl<const LEN:usize> ByteArray<LEN> {

    pub fn new() -> Self {
        Self {
            data: core::array::from_fn(|_| 0 ),
            head: 0,
            tail: 0
        }
    }

    pub fn push(&mut self, value:u8) {
        self.data[self.head] = value;
        self.head += 1
    }

    pub fn push_array(&mut self, arr:&[u8]) {
        for value in arr {
            self.push(*value);
        }
    }

    // pub fn pop_tail(&mut self) -> u8 {
    //     self.tail += 1;
    //     self.data[self.tail-1]
    // }

    pub fn is_empty(&self) -> bool {
        self.tail == self.head
    }

    // Ensures struct is completely filled before giving away its data array.
    pub fn validate_and_get_data(self) -> [u8; LEN] {
        if self.head == LEN {
            self.data
        } else {
            panic!("ByteArray validation error: Length should be {} but is {}", LEN, self.head)
        }
    }

}

impl<const LEN:usize> Default for ByteArray<LEN> {
    fn default() -> Self {
        Self::new()
    }
}
