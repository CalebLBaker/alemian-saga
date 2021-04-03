use alemian_saga_core::serialization;
use std::collections;

const LANGUAGES: [&'static str; 1] = ["english"];

#[derive(serde::Deserialize)]
enum JsonClass {
    Noble
}

fn json_class_to_class(json_class: JsonClass) -> serialization::Class {
    match json_class {
        JsonClass::Noble => serialization::Class::Noble
    }
}

#[derive(serde::Deserialize)]
struct JsonUnit {
    class: JsonClass,
    position: alemian_saga_core::Vector<serialization::MapDistance>,
}

#[allow(non_snake_case)]
#[derive(serde::Deserialize)]
#[serde(tag = "schema")]
enum JsonContent {
    Map {
        tileTypes: collections::HashMap<String, TileTypeInfo>,
        map: ndarray::Array2<String>,
        blue: Vec<JsonUnit>
    },
}

#[derive(serde::Deserialize)]
struct TileTypeInfo {
    image: String,
    move_cost: u32,
    defense: i32,
    evade: i32,
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
                JsonContent::Map { tileTypes, map, blue } => {
                    let mut name_to_index = collections::HashMap::new();
                    let out_blue = blue.into_iter().map(|j| {
                        serialization::Unit {
                            class: json_class_to_class(j.class),
                            position: j.position
                        }
                    }).collect::<Vec<_>>();
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
                                name: string_map.get(k).unwrap().clone(),
                                image: v.image.clone(),
                                defense: v.defense,
                                evade: v.evade,
                                move_cost: v.move_cost,
                            });
                        }
                        let new_map = serialization::Map {
                            tile_types: tile_types,
                            map: map.map(|x| *name_to_index.get(x).unwrap()),
                            blue: out_blue.clone()
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
