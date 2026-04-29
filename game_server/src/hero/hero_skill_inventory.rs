use super::hero_entity::HeroEntity;

pub const HERO_SKILL_ITEM_SIZE: usize = 3;

#[derive(Debug)]
#[derive(Clone)]
pub struct SkillData
{
    pub id : u8, //1
    pub rank : u8, // 1
    pub count : u8, // 1
}

impl SkillData
{
    pub fn to_bytes(&self) -> [u8; HERO_SKILL_ITEM_SIZE]
    {
        [self.id, self.rank, self.count]
    }
}

impl HeroEntity
{
    pub fn increase_skill_rank(&mut self, skill_id : u8, points : u8)
    {
        let mut found = false;
        for skill in &mut self.skills
        {
            if skill.id == skill_id
            {
                skill.rank += points;
                skill.count += 1;
                found = true;
                break;
            }
        }

        if !found
        {
            self.skills.push(SkillData { id: skill_id, rank: 1, count: 1 });
        }

        self.version += 1;
        self.inventory_version += 1;
    }

    pub fn reset_skill_counts(&mut self)
    {
        for skill in &mut self.skills
        {
            skill.count = 0;
        }
    }

    pub fn has_skill(&self, skill_id : u8) -> bool
    {
        self.skills.iter().any(|s| s.id == skill_id)
    }

    pub fn get_skill_rank(&self, skill_id : u8) -> Option<u8>
    {
        self.skills.iter().find(|s| s.id == skill_id).map(|s| s.rank)
    }

    pub fn get_skill_count(&self, skill_id : u8) -> Option<u8>
    {
        self.skills.iter().find(|s| s.id == skill_id).map(|s| s.count)
    }
}
