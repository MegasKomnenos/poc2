use serde::{ Serialize, Deserialize, };

#[derive(Serialize, Deserialize)]
pub struct AssetExtractableData {
    pub name: String,
    pub outs: [u8; 1],
    pub duration: u16,
}

#[derive(Serialize, Deserialize)]
pub struct AssetItemData {
    pub name: String,
}