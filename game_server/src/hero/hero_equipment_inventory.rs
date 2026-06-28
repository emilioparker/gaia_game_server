use super::hero_entity::HeroEntity;

pub const HERO_EQUIPMENT_INVENTORY_ITEM_SIZE: usize = 7;

pub struct EquipmentSlot(pub u32);

impl EquipmentSlot 
{
    pub const NOT_EQUIPPED:      u32 = 0;
    pub const ARMOR:             u32 = 1;
    pub const RIGHT_HAND_WEAPON: u32 = 2;
    pub const LEFT_HAND_WEAPON:  u32 = 3;
    pub const DUAL_HAND_WEAPON:  u32 = 4; 
}

#[derive(Debug)]
#[derive(Clone)]
pub struct EquipmentItem
{
    pub equipment_definition_id : u16, //2
    pub equipment_unique_id : u32, //4
    pub slot : u8, // 1 // this can be used to know where it is equipped. 0 means not equipped, 1 or more means equipped and the actual value tells you how it is equipped.
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

    pub fn get_armor(&self) -> Option<EquipmentItem>
    {
        for item in &self.equipment_inventory
        {
            if item.slot == EquipmentSlot::ARMOR as u8
            {
                return Some(item.clone())
            }
        }

        None
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

    pub fn is_slot_occupied(&self, slot: u8) -> bool
    {
        let slot_u32 = slot as u32;
        match slot_u32
        {
            EquipmentSlot::ARMOR =>
            {
                self.equipment_inventory.iter().any(|i| i.slot == EquipmentSlot::ARMOR as u8)
            }
            EquipmentSlot::RIGHT_HAND_WEAPON =>
            {
                // blocked if a dual weapon is equipped, or a right-hand weapon is already in slot
                self.equipment_inventory.iter().any(|i|
                    i.slot == EquipmentSlot::RIGHT_HAND_WEAPON as u8 ||
                    i.slot == EquipmentSlot::DUAL_HAND_WEAPON as u8
                )
            }
            EquipmentSlot::LEFT_HAND_WEAPON =>
            {
                // blocked if a dual weapon is equipped, or a left-hand weapon is already in slot
                self.equipment_inventory.iter().any(|i|
                    i.slot == EquipmentSlot::LEFT_HAND_WEAPON as u8 ||
                    i.slot == EquipmentSlot::DUAL_HAND_WEAPON as u8
                )
            }
            EquipmentSlot::DUAL_HAND_WEAPON =>
            {
                // blocked if any right, left, or dual weapon is already equipped
                self.equipment_inventory.iter().any(|i|
                    i.slot == EquipmentSlot::RIGHT_HAND_WEAPON as u8 ||
                    i.slot == EquipmentSlot::LEFT_HAND_WEAPON as u8 ||
                    i.slot == EquipmentSlot::DUAL_HAND_WEAPON as u8
                )
            }
            _ => false,
        }
    }

    pub fn is_equipment_in_slot(&self, equipment_unique_id : u32, slot:u8) -> bool
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
        let mut equipped_definition_id: u16 = 0;
        for item in self.equipment_inventory.iter_mut()
        {
            if item.equipment_unique_id == equipment_unique_id && item.slot == current_slot
            {
                equipped_definition_id = item.equipment_definition_id;
                item.slot = slot;
                successfuly_updated = true;
                break;
            }
        }

        if successfuly_updated
        {
            // clear the field that was occupied by current_slot
            match current_slot as u32
            {
                EquipmentSlot::ARMOR            => { self.armor = 0; }
                EquipmentSlot::RIGHT_HAND_WEAPON => { self.right_weapon = 0; }
                EquipmentSlot::LEFT_HAND_WEAPON  => { self.left_weapon = 0; }
                EquipmentSlot::DUAL_HAND_WEAPON  => { self.right_weapon = 0; }
                _ => {}
            }

            // set the field for the new slot
            match slot as u32
            {
                EquipmentSlot::ARMOR            => { self.armor = equipped_definition_id; }
                EquipmentSlot::RIGHT_HAND_WEAPON => { self.right_weapon = equipped_definition_id; }
                EquipmentSlot::LEFT_HAND_WEAPON  => { self.left_weapon = equipped_definition_id; }
                EquipmentSlot::DUAL_HAND_WEAPON  => { self.right_weapon = equipped_definition_id; }
                _ => {}
            }
        }

        return successfuly_updated;
    }
}
