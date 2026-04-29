use super::hero_entity::HeroEntity;


pub const HERO_EQUIPMENT_INVENTORY_ITEM_SIZE: usize = 7;


#[derive(Debug)]
#[derive(Clone)]
pub struct EquipmentItem
{
    pub equipment_definition_id : u16, //2
    pub equipment_unique_id : u32, //4
    pub slot : u8, // 1 // this can be used to know where it is equipped. 0 means not equipped, 1 means equipped.
}

impl EquipmentItem
{
    pub fn to_bytes(&self) -> [u8; HERO_EQUIPMENT_INVENTORY_ITEM_SIZE]
    {
        let mut start = 0;
        let mut buffer = [0u8;HERO_EQUIPMENT_INVENTORY_ITEM_SIZE];

        let equipment_definition_id_bytes = u16::to_le_bytes(self.equipment_definition_id); // 4 bytes
        let end = start + 2;
        buffer[start..end].copy_from_slice(&equipment_definition_id_bytes);
        start = end;

        let equipment_unique_id_bytes = u32::to_le_bytes(self.equipment_unique_id); // 4 bytes
        let end = start + 4;
        buffer[start..end].copy_from_slice(&equipment_unique_id_bytes);
        start = end;

        buffer[start] = self.slot;
        start += 1;

        buffer
    }
}

impl HeroEntity
{

    pub fn has_equipment(&self, unique_id : u32) -> bool
    {
        let mut found = false;
        for item in &self.equipment_inventory
        {
            if item.equipment_unique_id == unique_id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn has_equipment_of_type(&self, definition_id : u16) -> bool
    {
        let mut found = false;
        for item in &self.equipment_inventory
        {
            if item.equipment_definition_id == definition_id
            {
                found = true;
            }
        }
        return found;
    }

    pub fn add_equipment(&mut self, new_item : EquipmentItem)
    {
        self.equipment_inventory.push(new_item);
        self.version += 1;
        self.inventory_version += 1;
    }

    pub fn remove_equipment(&mut self, unique_id : u32) -> bool
    {
        let mut successfuly_removed = false;
        for (index, item) in &mut self.equipment_inventory.iter_mut().enumerate()
        {
            if item.equipment_unique_id == unique_id
            {
                self.equipment_inventory.swap_remove(index);
                successfuly_removed = true;
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

    pub fn is_slot_occupied(&mut self, slot:u8) -> bool
    {
        self.equipment_inventory.iter().any(|i| i.slot == slot)
    }

    pub fn is_equipment_in_slot(&mut self, equipment_unique_id : u32, slot:u8) -> bool
    {
        self.equipment_inventory.iter().any(|i| i.equipment_unique_id == equipment_unique_id && i.slot == slot)
    }

    pub fn equip_equipment(&mut self, equipment_unique_id : u32, current_slot : u8, slot: u8) -> bool
    {
        let occupied = self.is_slot_occupied(slot);
        if slot > 0 && occupied
        {
            cli_log::info!("-- max equip count reached for equipment");
            return false;
        }

        let mut successfuly_updated = false;
        for (index, item) in &mut self.equipment_inventory.iter_mut().enumerate()
        {
            if item.equipment_unique_id == equipment_unique_id && item.slot == current_slot
            {
                item.slot = slot;
                successfuly_updated = true;
                break;
            }
        }

        return successfuly_updated;
    }
}
