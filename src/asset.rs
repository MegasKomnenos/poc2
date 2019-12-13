use serde::{ Serialize, Deserialize, };

#[derive(Serialize, Deserialize)]
pub struct AssetWorkplaceData {
    pub name: String,
    pub inputs: [u8; 3],
    pub outs: [u8; 3],
    pub duration: u32,
}

#[derive(Serialize, Deserialize)]
pub struct AssetItemData {
    pub name: String,
}