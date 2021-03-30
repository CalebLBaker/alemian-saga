#[derive(serde::Deserialize)]
#[allow(non_snake_case)]
pub struct Keybindings {
    #[serde(default)]
    pub Right: Vec<String>,
    #[serde(default)]
    pub Left: Vec<String>,
    #[serde(default)]
    pub Up: Vec<String>,
    #[serde(default)]
    pub Down: Vec<String>,
    #[serde(default)]
    pub ZoomIn: Vec<String>,
    #[serde(default)]
    pub ZoomOut: Vec<String>,
}

