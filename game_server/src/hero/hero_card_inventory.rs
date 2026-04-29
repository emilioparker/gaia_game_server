use super::hero_entity::HeroEntity;


pub const HERO_CARD_INVENTORY_ITEM_SIZE: usize = 7;


#[derive(Debug)]
#[derive(Clone)]
pub struct CardItem
{
    pub card_definition_id : u16, //2
    pub card_unique_id : u32,//4
    pub slot : u8, // 1 // this can be used to know where it is equipped. 0 means not equipped, 1 means equipped.
}

impl CardItem 
{
    pub fn to_bytes(&self) -> [u8; HERO_CARD_INVENTORY_ITEM_SIZE]
    {
        let mut start = 0;
        let mut buffer = [0u8;HERO_CARD_INVENTORY_ITEM_SIZE];

        let card_id_bytes = u16::to_le_bytes(self.card_definition_id); // 4 bytes
        let end = start + 2; 
        buffer[start..end].copy_from_slice(&card_id_bytes);
        start = end;
        
        let card_unique_id_bytes = u32::to_le_bytes(self.card_unique_id); // 4 bytes
        let end = start + 4; 
        buffer[start..end].copy_from_slice(&card_unique_id_bytes);
        start = end;

        buffer[start] = self.slot;
        start += 1;

        // let end = start + 2; 
        // let amount_bytes = u16::to_le_bytes(self.amount); // 2 bytes
        // buffer[start..end].copy_from_slice(&amount_bytes);
        buffer
    }
}

impl HeroEntity
{

    pub fn has_card(&self, definition_id : u16) -> bool
    {
        let mut found = false;
        for item in &self.card_inventory 
        {
            if item.card_definition_id == definition_id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn add_card(&mut self, new_item : CardItem)
    {
        self.card_inventory.push(new_item);
        self.version += 1;
        self.inventory_version += 1;
    }

    pub fn remove_card(&mut self, card_unique_id : u32) -> bool
    {
        let mut successfuly_removed = false;
        for (index, item) in &mut self.card_inventory.iter_mut().enumerate() 
        {
            if item.card_unique_id == card_unique_id
            {
                successfuly_removed = true;
                self.card_inventory.swap_remove(index);
                break;
            }
        }

        if successfuly_removed 
        {
            self.inventory_version += 1;
            self.version += 1;
        }
        successfuly_removed
    }

    pub fn count_cards_in_slot(&mut self, slot:u8) -> usize
    {
        self.card_inventory.iter().filter(|i| i.slot == slot).count()
    }

    pub fn is_card_in_slot(&mut self, card_unique_id : u32, slot:u8) -> bool
    {
        self.card_inventory.iter().any(|i| i.card_unique_id == card_unique_id && i.slot == slot)
    }

    pub fn equip_card(&mut self, card_id : u32, current_slot : u8, slot: u8) -> bool
    {
        let equip_count = self.count_cards_in_slot(slot);
        if slot == 1 && equip_count >= 10
        {
            cli_log::info!("-- max equip count reached");
            return false;
        }

        let card_in_slot = self.is_card_in_slot(card_id, slot);
        if slot == 1 && card_in_slot
        {
            cli_log::info!("-- card of {card_id} is already equipped");
            return false;
        }

        for item in &mut self.card_inventory.iter_mut()
        {
            if item.card_unique_id == card_id && item.slot == current_slot
            {
                item.slot = slot;
                self.inventory_version += 1;
                self.version += 1;
                return true;
            }
        }

        return false;
    }
}