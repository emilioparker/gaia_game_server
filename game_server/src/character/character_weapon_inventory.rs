use super::character_entity::CharacterEntity;


pub const CHARACTER_WEAPON_INVENTORY_ITEM_SIZE: usize = 7;


#[derive(Debug)]
#[derive(Clone)]
pub struct WeaponItem
{
    pub weapon_id : u32, //4
    pub equipped : u8, // 1 // this can be used to know where it is equipped. 0 means not equipped, 1 means equipped.
    pub amount : u16 // 2
}

impl WeaponItem 
{
    pub fn to_bytes(&self) -> [u8; CHARACTER_WEAPON_INVENTORY_ITEM_SIZE]
    {
        let mut start = 0;
        let mut buffer = [0u8;CHARACTER_WEAPON_INVENTORY_ITEM_SIZE];
        let card_id_bytes = u32::to_le_bytes(self.weapon_id); // 4 bytes
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

impl CharacterEntity
{

    pub fn has_weapon(&self, id : u32) -> bool
    {
        let mut found = false;
        for item in &self.weapon_inventory 
        {
            if item.weapon_id == id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn add_weapon(&mut self, new_item : WeaponItem)
    {
        let mut found = false;
        for item in &mut self.weapon_inventory 
        {
            if item.weapon_id == new_item.weapon_id && item.equipped == new_item.equipped 
            {
                item.amount += new_item.amount;
                found = true;
            }
        }

        if !found 
        {
            self.weapon_inventory.push(new_item);
        }

        self.version += 1;
        self.inventory_version += 1;
    }

    pub fn remove_weapon(&mut self, old_item : WeaponItem) -> bool
    {
        let mut successfuly_removed = false;
        for (index, item) in &mut self.weapon_inventory.iter_mut().enumerate() 
        {
            if item.weapon_id == old_item.weapon_id && item.equipped == old_item.equipped
            {
                if item.amount >= old_item.amount
                {
                    item.amount -= old_item.amount;
                    successfuly_removed = true;
                }

                if item.amount == 0 
                {
                    self.weapon_inventory.swap_remove(index);
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

    pub fn count_weapons_in_slot(&mut self, slot:u8) -> usize
    {
        self.weapon_inventory.iter().filter(|i| i.equipped == slot).count()
    }

    pub fn count_weapon_in_slot_by_id(&mut self, weapon_id : u32, slot:u8) -> usize
    {
        self.weapon_inventory.iter().filter(|i| i.weapon_id == weapon_id && i.equipped == slot).count()
    }

    pub fn equip_weapon(&mut self, weapon_id : u32, current_slot : u8, slot: u8) -> bool
    {
        let equip_count = self.count_weapons_in_slot(slot);
        if slot > 0 && equip_count > 0 
        {
            println!("-- max equip count reached for weapons");
            return false;
        }

        let mut successfuly_removed = false;
        for (index, item) in &mut self.weapon_inventory.iter_mut().enumerate() 
        {
            if item.weapon_id == weapon_id && item.equipped == current_slot
            {
                if item.amount > 0
                {
                    item.amount -= 1;
                    successfuly_removed = true;
                }

                if item.amount == 0 
                {
                    self.weapon_inventory.swap_remove(index);
                }
                break;
            }
        }


        if successfuly_removed 
        {
            self.add_weapon(WeaponItem { weapon_id, equipped: slot, amount: 1 });
            if slot == 0
            {
                self.weapon = 0;
            }
            else
            {
                self.weapon = weapon_id as u8;
            }
            self.inventory_version += 1;
            self.version += 1;
        }
        successfuly_removed
    }
}