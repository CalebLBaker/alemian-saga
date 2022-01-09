use crate::*;
use numeric_types::*;

pub struct Unit {
  pub name: String,
  class: serialization::Class,
  pub remaining_move: MapDistance,
  pub level: Level,
  pub hp: HitPoints
}

impl From<serialization::Unit> for Unit {
    fn from(src: serialization::Unit) -> Self {
        Self {
            name: src.name,
            remaining_move: src.remaining_move,
            level: src.level,
            hp: src.hp
        }
    }
}

