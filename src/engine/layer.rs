use slotmap::{new_key_type, SecondaryMap, SlotMap};
use crate::*;

new_key_type! {
    /// A key to the World slotmap containing entities.
    pub struct LayerID;
}

// #[derive(Default, Clone, Copy)]
// #[repr(u8)]
// pub enum LayerID {
//     // #[default] Bg,
//     SceneBG,
//     SceneSprites,
//     #[default] Sprites,
//     Ui,
//     Count
// }
// implement_byte_id!(LayerID);

#[derive(Default)]
pub struct Layer {
    pub id: LayerID,
    // pub palette: u8,
    pub data: SecondaryMap<EntityID, Entity>,
}


#[derive(Default)]
pub struct LayerPool {
    pub(crate) entities:SlotMap<EntityID, LayerID>,    // Stores just the layer where each entity is
    pub(crate) layers: SlotMap<LayerID, Layer>,        // Each layer contains the actual entity
}


// Simplifies the process of getting the Layer from the EntityID, and then getting the Entity.
// Also helps keeping both slotmaps in sync.
impl LayerPool {

    pub fn get(&self, id:EntityID) -> Option<&Entity> {
        let layer_id = self.entities.get(id)?;
        self.layers[*layer_id].data.get(id)   
    }


    pub fn get_mut(&mut self, id:EntityID) -> Option<&mut Entity> {
        let layer_id = self.entities.get_mut(id)?;
        self.layers[*layer_id].data.get_mut(id)   
    }


    pub fn insert_layer(&mut self) -> LayerID {
        let key = self.layers.insert_with_key(|id|{
            Layer{
                id,
                // palette: palette.to_u8(),
                data: Default::default(),
            }
        });
        #[cfg(std)]{ println!("New layer with key {:?}", key); }
        key
    }


    pub fn insert_entity(&mut self, layer_id:LayerID) -> &mut Entity {
        if let Some(layer) = self.layers.get_mut(layer_id) {
            // Key generated when inserting the "layer_id" into self.entities
            // is re-used to insert the actual Entity into the Layer! Phew...
            let key = self.entities.insert_with_key(|key|{
                layer.data.insert(key, Entity::new(key));
                layer_id
            });
            
            return &mut layer.data[key]
        }
        panic!("Invalid LayerID: {:?}", layer_id)
    }


    pub fn remove_entity(&mut self, id:EntityID) {
        if let Some(layer_id) = self.entities.get(id) {
            self.layers[*layer_id].data.remove(id);
            self.entities.remove(id);
        }
    }


    // Clears all contents in the layer, but does not remove it
    pub fn clear_layer(&mut self, layer_id:LayerID) {
        if let Some(layer) = self.layers.get_mut(layer_id) {
            for entity_id in layer.data.keys() {
                self.entities.remove(entity_id);
            }
            layer.data.clear();
        }
    }
    
}