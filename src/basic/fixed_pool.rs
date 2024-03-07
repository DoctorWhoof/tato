use core::fmt::Error;

pub struct FixedPool<T, const CAP:usize>
where T:Default + Clone {
    data:[T; CAP],
    head:usize,
    spawn_head:usize,
}

impl<T, const CAP:usize> Default for FixedPool<T, CAP>
where T:Default + Clone {
    fn default() -> Self {
        Self {
            data: core::array::from_fn( |_| Default::default() ),
            head: 0,
            spawn_head: 0,
        }
    }
}

impl<T, const CAP:usize> FixedPool<T, CAP>
where T:Default + Clone {


    pub fn new() -> Self {
        Default::default()
    }


    pub fn len(&self) -> usize {
        self.head
    }


    pub fn is_empty(&self) -> bool {
        self.head == 0
    }


    pub fn clear(&mut self) {
        self.head = 0;
        self.spawn_head = 0;
    }


    // Loops around, instead of crashing beyond the capacity
    pub fn spawn(&mut self) -> Option<&T> {
        let mut result = None;
        if !self.is_empty() {
            result = Some(&self.data[self.spawn_head]);
            self.spawn_head += 1;
            if self.spawn_head == self.head { self.spawn_head = 0 }
        }
        result
    }


    /// Pushes a new item to the head of the buffer. Returns an error if capacity was exceeded.
    pub fn push(&mut self, elem:T) -> Result<(), &str> {
        if self.head == CAP {
            return Result::Err("Pool Error: Capacity exceeded (not a ring pool.");
        }
        self.data[self.head] = elem;
        self.head += 1;
        Ok(())
    }

    /// "Pops" the most recent item, reducing the length and returning the popped item.
    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty(){ return None }
        let result = Some(self.data[self.head].clone());
        self.head -= 1;
        if self.spawn_head >= self.head { self.spawn_head = 0 }
        result
    }


    pub fn get(&self, i:usize) -> Option<&T> {
        if i >= self.len() { return None }
        Some(&self.data[i])
    }


    pub fn get_mut(&mut self, i:usize) -> Option<&mut T> {
        if i >= self.len() { return None }
        Some(&mut self.data[i])
    }


    pub fn capacity(&self) -> usize {
        CAP
    }


    pub fn iter(&self) -> core::slice::Iter<T> {
        self.data[..self.head].iter()
    }


    pub fn iter_mut(&mut self) -> core::slice::IterMut<T> {
        self.data[..self.head].iter_mut()
    }


    /// Iterates through all items, only keeping the ones where "func" returns true
    pub fn retain(&mut self, mut func:impl FnMut(&T) -> bool){
        if self.is_empty() { return };
        let mut i = self.head - 1;
        loop {
            // Remove item if false
            if !func(&self.data[i]) {                 
                if self.len() > 1 {
                    // Promote item at tail (already updated with "func") to current index
                    self.data[i] = self.data[self.head - 1].clone();
                } 
                self.head -= 1;
            }
            if i == 0 {
                break;
            } else {
                i -= 1;
            }
        }
    }

}


