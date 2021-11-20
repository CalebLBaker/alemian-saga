use alemian_saga_core::numeric_types::*;
use alemian_saga_core::serialization;
use std::collections;

const LANGUAGES: [&str; 1] = ["english"];

#[derive(Clone, Copy, serde::Deserialize)]
enum JsonClass {
    Noble,
}

fn json_class_to_class(json_class: JsonClass) -> serialization::Class {
    match json_class {
        JsonClass::Noble => serialization::Class::Noble,
    }
}

#[derive(serde::Deserialize)]
struct JsonUnit {
    class: JsonClass,
    name: String,
    level: Level,
    hp: HitPoints,
    position: alemian_saga_core::Vector<serialization::MapDistance>,
    movement: MapDistance,
    remaining_move: MapDistance,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize)]
#[serde(tag = "schema")]
enum JsonContent {
    Map {
        tileTypes: collections::HashMap<String, TileTypeInfo>,
        map: ndarray::Array2<String>,
        blue: Vec<JsonUnit>,
    },
}

#[derive(serde::Deserialize)]
struct TileTypeInfo {
    image: String,
    move_cost: MapDistance,
    defense: HitPoints,
    evade: AccuracyPoints,
}

#[allow(non_snake_case)]
fn main() {
    let out_folder = std::path::Path::new("../generated-files");
    let _ = std::fs::create_dir(out_folder);
    for f in std::fs::read_dir("../../json-files").unwrap() {
        let file = f.unwrap();
        if file.file_type().unwrap().is_file() {
            let mut path = file.path();
            let reader = std::io::BufReader::new(std::fs::File::open(&path).unwrap());
            let json: JsonContent = serde_json::from_reader(reader).unwrap();
            match json {
                JsonContent::Map {
                    tileTypes,
                    map,
                    blue,
                } => {
                    let mut name_to_index = collections::HashMap::new();
                    let out_blue = blue
                        .iter()
                        .map(|j| serialization::Unit {
                            class: json_class_to_class(j.class),
                            name: j.name.as_str(),
                            hp: j.hp,
                            level: j.level,
                            position: j.position,
                            movement: j.movement,
                            remaining_move: j.remaining_move,
                        })
                        .collect::<Vec<_>>();
                    for l in LANGUAGES.iter() {
                        let lang_file =
                            std::fs::File::open(&format!("../../language/{}.json", l)).unwrap();
                        let lang_reader = std::io::BufReader::new(lang_file);
                        let string_map: collections::HashMap<String, String> =
                            serde_json::from_reader(lang_reader).unwrap();
                        let mut tile_types = vec![];
                        for (i, (k, v)) in tileTypes.iter().enumerate() {
                            name_to_index.insert(k.clone(), i as u32);
                            tile_types.push(serialization::TileType {
                                name: string_map.get(k).unwrap().as_str(),
                                image: v.image.as_str(),
                                defense: v.defense,
                                evade: v.evade,
                                move_cost: v.move_cost,
                            });
                        }
                        let new_map = serialization::Map {
                            tile_types,
                            map: map.map(|x| *name_to_index.get(x).unwrap()),
                            blue: out_blue.clone(),
                        };
                        path.set_extension("map");
                        let out_path = out_folder.join(l).join(path.file_name().unwrap());
                        let _ = std::fs::create_dir(out_folder.join(l));
                        let mut out_file = std::fs::File::create(out_path).unwrap();
                        rmp_serde::encode::write(&mut out_file, &new_map).unwrap();
                    }
                }
            }
        }
    }
}
