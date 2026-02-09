use super::hero_entity::HeroEntity;

pub const HERO_SKILL_ITEM_SIZE: usize = 2;

#[derive(Debug)]
#[derive(Clone)]
pub struct SkillData
{
    pub id : u8, //1
    pub rank : u8, // 1
}

impl SkillData
{
    pub fn to_bytes(&self) -> [u8; HERO_SKILL_ITEM_SIZE]
    {
        let buffer = [self.id, self.rank];
        buffer
    }
}

impl HeroEntity
{
    pub fn add_skill(&mut self, new_skill : SkillData)
    {
        let mut found = false;
        for skill in &mut self.skills
        {
            if skill.id == new_skill.id
            {
                skill.rank = new_skill.rank;
                found = true;
            }
        }

        if !found
        {
            self.skills.push(new_skill);
        }

        self.version += 1;
        self.inventory_version += 1;
    }

    pub fn remove_skill(&mut self, skill_id : u8) -> bool
    {
        let initial_len = self.skills.len();
        self.skills.retain(|s| s.id != skill_id);

        if self.skills.len() < initial_len
        {
            self.inventory_version += 1;
            self.version += 1;
            return true;
        }
        false
    }

    pub fn has_skill(&self, skill_id : u8) -> bool
    {
        self.skills.iter().any(|s| s.id == skill_id)
    }

    pub fn get_skill_rank(&self, skill_id : u8) -> Option<u8>
    {
        self.skills.iter().find(|s| s.id == skill_id).map(|s| s.rank)
    }
}
