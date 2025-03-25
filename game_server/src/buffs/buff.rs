use hyper::body::Buf;

use crate::{definitions::definitions_container::Definitions, long_term_storage_service::db_hero::StoredBuff};

pub const BUFF_STRENGTH: &str = "str";
pub const BUFF_DEFENSE: &str = "def";
pub const BUFF_INTELLIGENCE: &str = "int";
pub const BUFF_MANA: &str = "mana";
pub const BUFF_STATUS: &str = "status";

#[derive(Debug)]
#[derive(Clone)]
pub struct Buff
{
    pub buff_id : u8, //1
    pub hits: u8,// 1
    pub expiration_time:u32 //4
}

impl From<StoredBuff> for Buff
{
    fn from(buff: StoredBuff) -> Self
    {
        Buff {buff_id : buff.buff_id, hits : buff.hits, expiration_time : buff.expiration_time}
    }
}


pub trait BuffUser
{
    fn get_buffs_mut(&mut self) -> &mut Vec<Buff>;
    fn get_buffs(&self) -> &Vec<Buff>;
    fn set_buffs(&mut self, new_buffs: Vec<Buff>);
    fn get_buff_summary(&mut self) -> &mut [u8;5]; // this one is serialized but not saved 10 bytes
    fn has_buff(&self, buff_id : u8) -> bool
    {
        let mut found = false;
        for buff in self.get_buffs() 
        {
            if buff.buff_id == buff_id
            {
                found = true;
            }
        }
        return found;
    }

    fn add_buff(&mut self, buff_id:u8, current_time_in_seconds : u32, definitions: &Definitions) -> bool
    {
        cli_log::info!("---- add buff {buff_id}");
        if self.has_buff(buff_id) 
        {
            return false;
        }

        if let Some(buff) = definitions.get_buff_by_code(buff_id)
        {
            self.get_buffs_mut().push(Buff
            {
                buff_id,
                hits: buff.hits,
                expiration_time: current_time_in_seconds + buff.duration as u32,
            });

            self.summarize_buffs();
        }

        return true;
    }

    fn summarize_buffs(&mut self)
    {
        let mut values = Vec::new();
        for buff in self.get_buffs().iter()
        {
            let value = buff.buff_id;
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
                *value = 0u8;//(buff.card_id, buff.hits);
            }
            index += 1;
        }
    }

    fn use_buffs(&mut self, used_stats : Vec<&str>, definitions: &Definitions)
    {
        self.get_buffs_mut().iter_mut()
        .filter(|b| 
            {
                if let Some(def) = definitions.get_buff_by_code(b.buff_id)
                {
                    return used_stats.iter().any(|a| a == &&def.buff_type);
                }
                return false;
            })
        .for_each(|b| b.hits = b.hits.saturating_sub(1));
        let updated_buffs : Vec<Buff> = self.get_buffs().iter().filter(|b| b.hits > 0).map(|b| b.clone()).collect();
        self.set_buffs(updated_buffs);
        self.summarize_buffs();
    }

    fn removed_expired_buffs(&mut self, current_time_in_seconds:u32)
    {
        // cli_log::info!("--- removing expired buff");
        self.get_buffs_mut()
        .retain(|b| 
            {
                return current_time_in_seconds <  b.expiration_time;
            });

        self.summarize_buffs();
    }

    fn has_expired_buffs(&mut self, current_time_in_seconds:u32) -> bool
    {
        let active_buffs = self.get_buffs()
        .iter().filter(|x| {
            x.buff_id != 0 && x.expiration_time < current_time_in_seconds
        }).count();

        return active_buffs > 0;
    }
}

// impl Stat
// {
//     pub fn to_byte(&self) -> u8
//     {
//         let stat = match self
//         {
//             Stat::Strength => 0,
//             Stat::Defense => 1,
//             Stat::Intelligence => 2,
//             Stat::Mana => 3,
//         };

//         stat as u8
//     }

//     pub fn from_byte(data :u8) -> Stat
//     {
//         let stat = match data
//         {
//             0 => Stat::Strength,
//             1 => Stat::Defense,
//             2 => Stat::Intelligence,
//             3 => Stat::Mana,
//             _ => Stat::Mana,
//         };
//         stat
//     }
// }