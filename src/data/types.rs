use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Skins an items
#[derive(Debug, Deserialize, Serialize)]
pub struct SkinVariation {
    formatted_name: String,
    condition_images: [String; 5],
    inspect_urls: [String; 5],
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Skin {
    pub formatted_name: String,
    pub description: String,
    pub flavor_text: String,
    pub quality: String,
    pub inspect_urls: [String; 5],
    pub image_urls: [String; 5],
    pub stattrak_available: bool,
    pub souvenir_available: bool,
    pub containers_found_in: Vec<String>,
    pub weapon_type: String,
    pub min_float: String,
    pub max_float: String,
    pub worst_condition_index: usize,
    pub best_condition_index: usize,
    pub variations: HashMap<String, SkinVariation>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Sticker {
    pub formatted_name: String,
    pub description: String,
    pub flavor_text: String,
    pub quality: String,
    pub inspect_urls: [String; 1],
    pub image_urls: [String; 1],
    pub stattrak_available: bool,
    pub souvenir_available: bool,
    pub containers_found_in: Vec<String>,
}

#[derive(Debug)]
pub enum Item {
    Skin(Skin),
    Sticker(Sticker),
}

// Containers
#[derive(Debug, Deserialize, Serialize)]
pub struct Container {
    pub formatted_name: String,
    pub image_url: String,
    pub items: HashMap<String, Vec<String>>,
    pub requires_key: bool,
}

// Game data
#[derive(Debug)]
pub struct GameData {
    pub item_data: HashMap<String, Item>,
    pub item_autocomplete_data: AutocompleteData,
    pub skin_unformatted_names: Vec<String>,
    pub container_data: HashMap<String, Container>,
    pub container_autocomplete_data: AutocompleteData,
}

#[derive(Debug, Default)]
pub struct AutocompleteData {
    pub formatted: Vec<String>,
    pub unformatted: Vec<String>,
}
