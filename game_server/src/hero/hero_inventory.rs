use rand::rngs::StdRng;

use crate::hero::hero_card_inventory::CardItem;
use crate::definitions::definitions_container::Definitions;

use super::hero_entity::HeroEntity;

pub const HERO_INVENTORY_ITEM_SIZE: usize = 6;

#[derive(Debug)]
#[derive(Clone)]
pub struct InventoryItem
{
    pub item_id : u32, //4
    pub amount : u16 // 2
}

impl InventoryItem 
{
    pub fn to_bytes(&self) -> [u8; HERO_INVENTORY_ITEM_SIZE]
    {
        let mut start = 0;
        let mut buffer = [0u8;HERO_INVENTORY_ITEM_SIZE];
        let item_id_bytes = u32::to_le_bytes(self.item_id); // 4 bytes
        let end = start + 4; 
        buffer[start..end].copy_from_slice(&item_id_bytes);
        start = end;

        let end = start + 2; 
        let amount_bytes = u16::to_le_bytes(self.amount); // 2 bytes
        buffer[start..end].copy_from_slice(&amount_bytes);
        buffer
    }
}

impl HeroEntity 
{
    pub fn has_inventory_item(&self, id : u32) -> bool
    {
        let mut found = false;
        for item in &self.inventory 
        {
            if item.item_id == id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn add_inventory_item(&mut self, new_item : InventoryItem)
    {
        let mut found = false;
        for item in &mut self.inventory 
        {
            if item.item_id == new_item.item_id
            {
                item.amount += new_item.amount;
                found = true;
            }
        }

        if !found 
        {
            self.inventory.push(new_item);
            self.version += 1;
        }

        self.inventory_version += 1;
    }

    pub fn remove_inventory_item(&mut self, old_item : InventoryItem) -> bool
    {
        let mut successfuly_removed = false;
        for (index, item) in &mut self.inventory.iter_mut().enumerate() 
        {
            if item.item_id == old_item.item_id
            {
                if item.amount >= old_item.amount
                {
                    item.amount -= old_item.amount;
                    successfuly_removed = true;
                }

                if item.amount == 0 
                {
                    self.inventory.swap_remove(index);
                }
                break;
            }
        }

        if successfuly_removed {
            self.inventory_version += 1;
            self.version += 1;
        }
        successfuly_removed
    }

    pub fn craft_card(&mut self, definitions : &Definitions) -> bool
    {
        let set_size = self.inventory.iter().filter(|i| i.item_id >= 6 && i.item_id <= 20).count();
        cli_log::info!("---- complete set {set_size}");

        // 6 to 20 are the ids of the cards... this is very very hard coded stuff.
        if set_size == 15 
        {
            for id in 6..=20
            {
                self.remove_inventory_item(InventoryItem { item_id: id, amount: 1 });
            }

            let cards_count = definitions.cards.len();

            let mut random_generator = <StdRng as rand::SeedableRng>::from_entropy();
            let x =  rand::Rng::gen::<f32>(&mut random_generator);
            let card_id = (x * cards_count as f32).floor() as u16;

            let new_id = self.card_id_generator + 1;
            self.card_id_generator += 1;

            self.add_card(CardItem
            {
                card_definition_id: card_id,
                slot: 0,
                card_unique_id: new_id,
            });

            return true;
        }

        false
    }
}