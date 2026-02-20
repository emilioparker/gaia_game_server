use super::hero_entity::HeroEntity;


pub const HERO_EQUIPMENT_INVENTORY_ITEM_SIZE: usize = 7;


#[derive(Debug)]
#[derive(Clone)]
pub struct EquipmentItem
{
    pub equipment_id : u32, //4
    pub equipped : u8, // 1 // this can be used to know where it is equipped. 0 means not equipped, 1 means equipped.
    pub amount : u16 // 2
}

impl EquipmentItem
{
    pub fn to_bytes(&self) -> [u8; HERO_EQUIPMENT_INVENTORY_ITEM_SIZE]
    {
        let mut start = 0;
        let mut buffer = [0u8;HERO_EQUIPMENT_INVENTORY_ITEM_SIZE];
        let equipment_id_bytes = u32::to_le_bytes(self.equipment_id); // 4 bytes
        let end = start + 4;
        buffer[start..end].copy_from_slice(&equipment_id_bytes);
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

    pub fn has_equipment(&self, id : u32) -> bool
    {
        let mut found = false;
        for item in &self.equipment_inventory
        {
            if item.equipment_id == id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn add_equipment(&mut self, new_item : EquipmentItem)
    {
        let mut found = false;
        for item in &mut self.equipment_inventory
        {
            if item.equipment_id == new_item.equipment_id && item.equipped == new_item.equipped
            {
                item.amount += new_item.amount;
                found = true;
            }
        }

        if !found
        {
            self.equipment_inventory.push(new_item);
        }

        self.version += 1;
        self.inventory_version += 1;
    }

    pub fn remove_equipment(&mut self, old_item : EquipmentItem) -> bool
    {
        let mut successfuly_removed = false;
        for (index, item) in &mut self.equipment_inventory.iter_mut().enumerate()
        {
            if item.equipment_id == old_item.equipment_id && item.equipped == old_item.equipped
            {
                if item.amount >= old_item.amount
                {
                    item.amount -= old_item.amount;
                    successfuly_removed = true;
                }

                if item.amount == 0
                {
                    self.equipment_inventory.swap_remove(index);
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

    pub fn count_equipment_in_slot(&mut self, slot:u8) -> usize
    {
        self.equipment_inventory.iter().filter(|i| i.equipped == slot).count()
    }

    pub fn count_equipment_in_slot_by_id(&mut self, equipment_id : u32, slot:u8) -> usize
    {
        self.equipment_inventory.iter().filter(|i| i.equipment_id == equipment_id && i.equipped == slot).count()
    }

    pub fn equip_equipment(&mut self, equipment_id : u32, current_slot : u8, slot: u8) -> bool
    {
        let equip_count = self.count_equipment_in_slot(slot);
        if slot > 0 && equip_count > 0
        {
            cli_log::info!("-- max equip count reached for equipment");
            return false;
        }

        let mut successfuly_removed = false;
        for (index, item) in &mut self.equipment_inventory.iter_mut().enumerate()
        {
            if item.equipment_id == equipment_id && item.equipped == current_slot
            {
                if item.amount > 0
                {
                    item.amount -= 1;
                    successfuly_removed = true;
                }

                if item.amount == 0
                {
                    self.equipment_inventory.swap_remove(index);
                }
                break;
            }
        }


        if successfuly_removed
        {
            self.add_equipment(EquipmentItem { equipment_id, equipped: slot, amount: 1 });
            if slot == 0
            {
                self.equipped_item = 0;
            }
            else
            {
                self.equipped_item = equipment_id as u8;
            }
            self.inventory_version += 1;
            self.version += 1;
        }
        successfuly_removed
    }
}
