use crate::definitions::definitions_container::Definitions;

#[derive(Clone, PartialEq, Debug)]
pub enum Stat
{
    Strength,
    Defense,
    Intelligence,
    Mana
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Buff
{
    pub card_id : u32, //1
    pub stat : Stat, //1
    pub buff_amount : f32, // 4
    pub hits: u8,// 1
    pub expiration_time:u32 //4
}


pub trait BuffUser
{
    fn get_buffs_mut(&mut self) -> &mut Vec<Buff>;
    fn get_buffs(&self) -> &Vec<Buff>;
    fn set_buffs(&mut self, new_buffs: Vec<Buff>);
    fn get_buff_summary(&mut self) -> &mut [(u8,u8);5]; // this one is serialized but not saved 10 bytes
    fn has_buff(&self, card_id : u32) -> bool
    {
        let mut found = false;
        for buff in self.get_buffs() 
        {
            if buff.card_id == card_id
            {
                found = true;
            }
        }
        return found;
    }

    fn add_buff(&mut self, card_id:u32, definitions: &Definitions) -> bool
    {
        if self.has_buff(card_id) 
        {
            return false;
        }

        if let Some(card) = definitions.get_card(card_id as usize)
        {
            if card.card_type == "passive"
            {
                if card.defense_factor > 0f32
                {
                    self.get_buffs_mut().push(Buff
                    {
                        card_id,
                        stat: Stat::Defense,
                        buff_amount: card.defense_factor,
                        hits: card.hits,
                        expiration_time: 100,
                    })
                }
                if card.strength_factor > 0f32
                {
                    self.get_buffs_mut().push(Buff
                    {
                        card_id,
                        stat: Stat::Strength,
                        buff_amount: card.strength_factor,
                        hits: card.hits,
                        expiration_time: 100,
                    })
                }
            }
            self.summarize_buffs();
        }

        return true;
    }

    fn summarize_buffs(&mut self)
    {
        // let mut buffs_summary : [(u8,u8);5]= [(0, 0),(0, 0), (0, 0), (0, 0), (0, 0)];
        let mut values = Vec::new();
        for buff in self.get_buffs().iter()
        {
            let value = ((buff.card_id - 10000) as u8, buff.hits);//(buff.card_id, buff.hits);
            values.push(value);
        }

        let mut index = 0;
        for value in self.get_buff_summary().iter_mut()
        {
            if let Some(buff) = values.get(index)
            {
                *value = *buff;//(buff.card_id, buff.hits);
            }
            else
            {
                *value = (0u8,0u8);//(buff.card_id, buff.hits);
            }
            index += 1;
        }
    }

    fn use_buffs(&mut self, used_stats : Vec<Stat>)
    {
        self.get_buffs_mut().iter_mut()
        .filter(|b| used_stats.contains(&b.stat))
        .for_each(|b| b.hits = b.hits.saturating_sub(1));
        let updated_buffs : Vec<Buff> = self.get_buffs().iter().filter(|b| b.hits > 0).map(|b| b.clone()).collect();
        self.set_buffs(updated_buffs);
        self.summarize_buffs();
    }

}

impl Stat
{
    pub fn to_byte(&self) -> u8
    {
        let stat = match self
        {
            Stat::Strength => 0,
            Stat::Defense => 1,
            Stat::Intelligence => 2,
            Stat::Mana => 3,
        };

        stat as u8
    }

    pub fn from_byte(data :u8) -> Stat
    {
        let stat = match data
        {
            0 => Stat::Strength,
            1 => Stat::Defense,
            2 => Stat::Intelligence,
            3 => Stat::Mana,
            _ => Stat::Mana,
        };
        stat
    }
}