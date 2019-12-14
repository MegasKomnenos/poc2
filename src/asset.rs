use serde::{ Serialize, Deserialize, };

#[derive(Serialize, Deserialize)]
pub struct AssetWorkplaceData {
    pub name: String,
    pub inputs: Vec<u8>,
    pub outs: Vec<u8>,
    pub duration: u32,
}

#[derive(Serialize, Deserialize)]
pub struct AssetItemData {
    pub name: String,
}