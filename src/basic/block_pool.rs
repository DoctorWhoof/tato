use core::marker::PhantomData;
use alloc::vec::Vec;

/// Manages assets within a fixed-size memory pool.
pub struct BlockPool<T>
where T: Clone {
    pub(crate) data: Vec<T>,
    pub(crate) blocks: Vec<Option<Block<T>>>,
    
    item_capacity: usize,
    item_head: usize,

    block_capacity: u8,
}


/// Represents metadata for managing an asset in a memory pool.
#[derive(Debug)]
pub struct Block<T> {
    pub start: usize,
    pub length: usize,
    head: usize,
    marker: PhantomData<T>
}


impl<T> BlockPool<T>
where T: Clone {

    pub fn new(item_capacity:usize, block_capacity:u8, default_item:T) -> Self {
        BlockPool {
            data: (0..item_capacity).map(|_| default_item.clone() ).collect(),
            blocks: (0..block_capacity).map(|_| None ).collect(),
            item_head: 0,
            item_capacity,
            block_capacity
        }
    }


    pub fn clear(&mut self) {
        for item in self.blocks.iter_mut(){
            *item = None
        }
        self.item_head = 0;
    }


    pub fn init_block(&mut self, index:u8, length:usize, default_value:T) -> Result<(), &'static str> {
        if (self.item_capacity - self.item_head) < length {
            return Err("BlockPool: Not enough space for new block")
        }

        if index >= self.block_capacity {
            return Err("BlockPool: Invalid Block index")
        }

        self.blocks[index as usize] = Some(
            Block{
                start: self.item_head,
                length,
                head: 0,
                marker: PhantomData
            }
        );
        self.item_head += length;

        if self.data.len() < self.item_head {
            self.data.resize(self.item_head, default_value.clone())
        }

        Ok(())
    }


    pub fn add_item_to_block(&mut self, block_index:usize, item:T) -> Result<(), &'static str> {

        let Some(ref mut block) = self.blocks[block_index] else {
            return Err("BlockPool: Attempt to insert data into non initialized block")
        };

        if block.head == block.length {
            return Err("BlockPool: Block capacity exceeded")
        }

        let item_index = block.start + block.head;
        self.data[item_index] = item;
        block.head += 1;

        Ok(())
    }


    fn get_item_index(&self, block_id:u8, item_id: usize) -> Option<usize> {
        if block_id >= self.block_capacity { return None }
        let block = self.blocks[block_id as usize].as_ref()?;
        if item_id >= block.length { return None }
        Some(block.start + item_id)
    }


    pub fn get(&self, block_id:u8, item_id: usize) -> Option<&T> {
        let index = self.get_item_index(block_id, item_id)?;
        Some(&self.data[index])
    }


    pub fn get_mut(&mut self, block_id:u8, item_id: usize) -> Option<&mut T> {
        let index = self.get_item_index(block_id, item_id)?;
        Some(&mut self.data[index])
    }


    pub fn get_block(&self, block_id:u8) -> &Option<Block<T>> {
        &self.blocks[block_id as usize]
    }


    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }


    /// Removes an asset by its ID.
    pub fn remove_block(&mut self, block_id:u8) -> Result<(), &'static str> {

        let Some(removed_block) = self.blocks[block_id as usize].take() else {
            return Err("BlockPool: Attempt to insert data into non initialized block")
        };

        self.item_head -= removed_block.length;

        // Update blocks above the removed block
        for block_option in self.blocks.iter_mut() {
            if let Some(other_block) = block_option.as_mut(){
                // Check if this block is "above" the removed block on the stack
                if other_block.start > removed_block.start {
                    let new_start = other_block.start - removed_block.length;

                    // let src = other_block.start as usize .. (other_block.start + other_block.length) as usize;
                    // self.data.copy_within(src, new_start as usize);
                    // self.data.clone_from_slice(src);

                    // TODO: Find a faster way that doesn't require Copy trait
                    for i in 0 .. other_block.length {
                        self.data[new_start + i ] = self.data[other_block.start + i].clone();
                    }
    
                    other_block.start = new_start;
                }
            }
        }



    //     let index = self.blocks.iter().position(|asset| asset.id == id)?;
    //     let asset = self.blocks.remove(index);

    //     // Shift data to fill the gap
    //     let next_start = asset.start + asset.length;
    //     let tail = &self.memory_pool[next_start..];
    //     let dest = &mut self.memory_pool[asset.start..];
    //     dest.copy_from_slice(tail);

    //     // Update subsequent asset positions
    //     for asset in &mut self.blocks {
    //         if asset.start > next_start {
    //             asset.start -= asset.length;
    //         }
    //     }

        Ok(())
    }
}
