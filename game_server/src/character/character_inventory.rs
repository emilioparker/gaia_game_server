use super::character_entity::CharacterEntity;

pub const CHARACTER_INVENTORY_ITEM_SIZE: usize = 7;

#[derive(Debug)]
#[derive(Clone)]
pub struct InventoryItem
{
    pub item_id : u32, //4
    pub equipped : u8, // 1 // this can be used to know where it is equipped. 0 means not equipped, 1 means equipped.
    pub amount : u16 // 2
}

impl InventoryItem 
{
    pub fn to_bytes(&self) -> [u8; CHARACTER_INVENTORY_ITEM_SIZE]
    {
        let mut start = 0;
        let mut buffer = [0u8;CHARACTER_INVENTORY_ITEM_SIZE];
        let item_id_bytes = u32::to_le_bytes(self.item_id); // 4 bytes
        let end = start + 4; 
        buffer[start..end].copy_from_slice(&item_id_bytes);
        start = end;

        buffer[start] = self.equipped;
        start += 1;

        let end = start + 2; 
        let amount_bytes = u16::to_le_bytes(self.amount); // 2 bytes
        buffer[start..end].copy_from_slice(&amount_bytes);
        buffer
    }
}

impl CharacterEntity 
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
            if item.item_id == new_item.item_id && item.equipped == new_item.equipped 
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
            if item.item_id == old_item.item_id && item.equipped == old_item.equipped
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

    pub fn count_items_in_slot(&mut self, slot:u8) -> usize
    {
        self.inventory.iter().filter(|i| i.equipped == slot).count()
    }

    pub fn equip_inventory_item(&mut self, item_id : u32, current_slot : u8, slot: u8) -> bool
    {
        let count = self.count_items_in_slot(slot);
        if slot == 1 && count >= 10
        {
            return false;
        }

        let mut successfuly_removed = false;
        for (index, item) in &mut self.inventory.iter_mut().enumerate() 
        {
            if item.item_id == item_id && item.equipped == current_slot
            {
                if item.amount > 0
                {
                    item.amount -= 1;
                    successfuly_removed = true;
                }

                if item.amount == 0 
                {
                    self.inventory.swap_remove(index);
                }
                break;
            }
        }


        if successfuly_removed 
        {
            self.add_inventory_item(InventoryItem { item_id, equipped: slot, amount: 1 });
            self.inventory_version += 1;
            self.version += 1;
        }
        successfuly_removed
    }
}