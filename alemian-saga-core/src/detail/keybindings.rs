#[derive(serde::Deserialize)]
#[allow(non_snake_case)]
pub struct Keybindings<'a> {
    #[serde(borrow, default)]
    pub Right: Vec<&'a str>,
    #[serde(default)]
    pub Left: Vec<&'a str>,
    #[serde(default)]
    pub Up: Vec<&'a str>,
    #[serde(default)]
    pub Down: Vec<&'a str>,
    #[serde(default)]
    pub ZoomIn: Vec<&'a str>,
    #[serde(default)]
    pub ZoomOut: Vec<&'a str>,
    #[serde(default)]
    pub Select: Vec<&'a str>,
}
