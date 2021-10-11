use core::marker::PhantomData;

#[macro_use]
mod map_distance;

#[macro_use]
mod hit_points;

#[macro_use]
mod accuracy_points;

#[macro_use]
mod level;

system! {
    quantities: Q {
        map_distance: tile, D;
        hit_points: point, H;
        accuracy_points: point, A;
        level: level, L;
    }
    units: U {
        mod map_distance::MapDistance,
        mod hit_points::HitPoints,
        mod accuracy_points::AccuracyPoints,
        mod level::Level,
    }
}

mod i32;
mod u32;

pub use self::i32::AccuracyPoints;
pub use self::i32::HitPoints;
pub use self::i32::MapDistance;
pub use self::u32::Level;

pub const fn map_dist(value: i32) -> MapDistance {
    MapDistance {
        dimension: PhantomData,
        units: PhantomData,
        value,
    }
}

pub const ZERO_TILES: MapDistance = map_dist(0);
pub const ONE_TILE: MapDistance = map_dist(1);

pub const fn hp(value: i32) -> HitPoints {
    HitPoints {
        dimension: PhantomData,
        units: PhantomData,
        value,
    }
}

pub const ZERO_HP: HitPoints = hp(0);

pub const fn accuracy_pts(value: i32) -> AccuracyPoints {
    AccuracyPoints {
        dimension: PhantomData,
        units: PhantomData,
        value,
    }
}

pub const BASE_EVADE_BONUS: AccuracyPoints = accuracy_pts(0);

pub const fn level(value: u32) -> Level {
    Level {
        dimension: PhantomData,
        units: PhantomData,
        value,
    }
}
