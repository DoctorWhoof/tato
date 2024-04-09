use tato::{EntityID, Vec2};


pub struct Grid {
    pub pos: Vec2<f32>,
    size: Vec2<u8>,
    slots: Vec<Vec<Option<EntityID>>>
}

impl Grid {

    pub fn new(x:f32, y:f32, cols:u8, rows:u8) -> Self {
        Grid {
            pos: Vec2::new(x, y),
            size: Vec2::new(cols, rows),
            slots: vec![ vec![None; rows as usize]; cols as usize]
        }
    }

    pub fn len(&self) -> usize {
        self.size.x as usize * self.size.y as usize
    }


    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }


    pub fn size(&self) -> Vec2<u8> {
        self.size
    }


    pub fn set(&mut self, col:u8, row:u8, id:EntityID) {
        if col >= self.size.x || row >= self.size.y {
            panic!("Grid: Error, capacity exceeded")
        }
        self.slots[col as usize][row as usize] = Some(id)
    }


    pub fn get(&mut self, col:u8, row:u8) -> Option<EntityID> {
        if col >= self.size.x || row >= self.size.y {
            panic!("Grid: Error, capacity exceeded")
        }
        self.slots[col as usize][row as usize]
    }


    pub fn clear(&mut self) {
        for col in self.slots.iter_mut() {
            for slot in col.iter_mut() {
                *slot = None;
            }
        }
    }


    pub fn slots(&self) -> &Vec<Vec<Option<EntityID>>> {
        &self.slots
    }
    
}