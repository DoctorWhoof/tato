
pub struct Pool<T, const CAP:usize>
where T:Default {
    data:[T; CAP],
    head:usize
}

impl<T, const CAP:usize> Default
for Pool<T, CAP>
where T:Default {
    fn default() -> Self {
        Self {
            data: core::array::from_fn( |_| Default::default() ),
            head: 0
        }
    }
}

impl<T, const CAP:usize> Pool<T, CAP>
where T:Default {

    pub fn push(&mut self, elem:T){
        if self.head == CAP { panic!("Pool Error: Capacity of {} exceeded", CAP) }
        self.data[self.head] = elem;
        self.head += 1;
    }


    pub fn get(&self, i:usize) -> Option<&T> {
        if i >= self.head { return None }
        Some(&self.data[i])
    }


    pub fn get_mut(&mut self, i:usize) -> Option<&mut T> {
        if i >= self.head { return None }
        Some(&mut self.data[i])
    }


    /// Expands pool to include index!
    pub fn insert(&mut self, elem:T, i:usize) {
        if i >= CAP { panic!("Pool Error: Capacity of {} exceeded", CAP) }
        if i >= self.head { self.head = i }
        self.data[i] = elem;
        self.head += 1;
    }


    pub fn len(&self) -> usize {
        self.head
    }

}
