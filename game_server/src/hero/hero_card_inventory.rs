use super::hero_entity::HeroEntity;


pub const HERO_CARD_INVENTORY_ITEM_SIZE: usize = 7;


#[derive(Debug)]
#[derive(Clone)]
pub struct CardItem
{
    pub card_id : u32, //4
    pub equipped : u8, // 1 // this can be used to know where it is equipped. 0 means not equipped, 1 means equipped.
    pub amount : u16 // 2
}

impl CardItem 
{
    pub fn to_bytes(&self) -> [u8; HERO_CARD_INVENTORY_ITEM_SIZE]
    {
        let mut start = 0;
        let mut buffer = [0u8;HERO_CARD_INVENTORY_ITEM_SIZE];
        let card_id_bytes = u32::to_le_bytes(self.card_id); // 4 bytes
        let end = start + 4; 
        buffer[start..end].copy_from_slice(&card_id_bytes);
        start = end;

        buffer[start] = self.equipped;
        start += 1;

        let end = start + 2; 
        let amount_bytes = u16::to_le_bytes(self.amount); // 2 bytes
        buffer[start..end].copy_from_slice(&amount_bytes);
        buffer
    }
}

impl HeroEntity
{

    pub fn has_card(&self, id : u32) -> bool
    {
        let mut found = false;
        for item in &self.card_inventory 
        {
            if item.card_id == id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn add_card(&mut self, new_item : CardItem)
    {
        let mut found = false;
        for item in &mut self.card_inventory 
        {
            if item.card_id == new_item.card_id && item.equipped == new_item.equipped 
            {
                item.amount += new_item.amount;
                found = true;
            }
        }

        if !found 
        {
            self.card_inventory.push(new_item);
        }

        self.version += 1;
        self.inventory_version += 1;
    }

    pub fn remove_card(&mut self, old_item : CardItem) -> bool
    {
        let mut successfuly_removed = false;
        for (index, item) in &mut self.card_inventory.iter_mut().enumerate() 
        {
            if item.card_id == old_item.card_id && item.equipped == old_item.equipped
            {
                if item.amount >= old_item.amount
                {
                    item.amount -= old_item.amount;
                    successfuly_removed = true;
                }

                if item.amount == 0 
                {
                    self.card_inventory.swap_remove(index);
                }
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
        self.card_inventory.iter().filter(|i| i.equipped == slot).count()
    }

    pub fn count_card_in_slot(&mut self, card_id : u32, slot:u8) -> usize
    {
        self.card_inventory.iter().filter(|i| i.card_id == card_id && i.equipped == slot).count()
    }

    pub fn equip_card(&mut self, card_id : u32, current_slot : u8, slot: u8) -> bool
    {
        let equip_count = self.count_cards_in_slot(slot);
        if slot == 1 && equip_count >= 10
        {
            cli_log::info!("-- max equip count reached");
            return false;
        }

        let equip_card_count = self.count_card_in_slot(card_id, slot);
        if slot == 1 && equip_card_count > 0 
        {
            cli_log::info!("-- card of {card_id} is already equipped");
            return false;
        }

        let mut successfuly_removed = false;
        for (index, item) in &mut self.card_inventory.iter_mut().enumerate() 
        {
            if item.card_id == card_id && item.equipped == current_slot
            {
                if item.amount > 0
                {
                    item.amount -= 1;
                    successfuly_removed = true;
                }

                if item.amount == 0 
                {
                    self.card_inventory.swap_remove(index);
                }
                break;
            }
        }


        if successfuly_removed 
        {
            self.add_card(CardItem { card_id, equipped: slot, amount: 1 });
            self.inventory_version += 1;
            self.version += 1;
        }
        successfuly_removed
    }
}