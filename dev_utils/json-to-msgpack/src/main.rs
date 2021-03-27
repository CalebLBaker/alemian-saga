use alemian_saga_core::serialization;

#[allow(non_snake_case)]
#[derive(serde::Deserialize)]
#[serde(tag = "schema")]
enum JsonContent {
    Map {
        tileTypes: std::collections::HashMap<String, TileTypeInfo>,
        map: ndarray::Array2<String>,
    },
}

#[derive(serde::Deserialize)]
struct TileTypeInfo {
    image: String,
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
                JsonContent::Map { tileTypes, map } => {
                    let mut name_to_index = std::collections::HashMap::new();
                    let mut tile_types = vec![];
                    for (i, (k, v)) in tileTypes.into_iter().enumerate() {
                        name_to_index.insert(k.clone(), i as u32);
                        tile_types.push(serialization::TileType {
                            name: k,
                            image: v.image,
                        });
                    }
                    let new_map = serialization::Map {
                        tile_types: tile_types,
                        map: map.map(|x| *name_to_index.get(x).unwrap()),
                    };
                    path.set_extension("map");
                    let out_path = out_folder.join(path.file_name().unwrap());
                    let mut out_file = std::fs::File::create(out_path).unwrap();
                    rmp_serde::encode::write(&mut out_file, &new_map).unwrap();
                }
            }
        }
    }
}
