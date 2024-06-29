use crate::data::types::{AutocompleteData, Container, GameData, Item, Skin, Sticker};
use reqwest::{Error, Response};
use std::collections::HashMap;

async fn fetch_data<T>(url: &str) -> Result<HashMap<String, T>, Error>
where
    T: serde::de::DeserializeOwned,
{
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    if response.status().is_success() {
        let json: HashMap<String, T> = response.json().await?;
        Ok(json)
    } else {
        Err(Response::error_for_status(response).unwrap_err())
    }
}

pub async fn fetch_game_data() -> Result<GameData, Error> {
    let mut item_data = HashMap::new();
    let mut item_autocomplete_data = AutocompleteData::default();
    let mut skin_unformatted_names = Vec::new();
    let skin_data =
        fetch_data::<Skin>("https://spacerulerwill.github.io/CS2-API/api/skins.json").await?;
    for (name, skin) in skin_data {
        item_autocomplete_data
            .formatted
            .push(skin.formatted_name.clone());
        item_autocomplete_data.unformatted.push(name.clone());
        skin_unformatted_names.push(name.clone());
        item_data.insert(name, Item::Skin(skin));
    }
    let sticker_data =
        fetch_data::<Sticker>("https://spacerulerwill.github.io/CS2-API/api/stickers.json").await?;
    for (name, sticker) in sticker_data {
        item_autocomplete_data
            .formatted
            .push(sticker.formatted_name.clone());
        item_autocomplete_data.unformatted.push(name.clone());
        item_data.insert(name, Item::Sticker(sticker));
    }
    let container_data: HashMap<String, Container> =
        fetch_data::<Container>("https://spacerulerwill.github.io/CS2-API/api/cases.json").await?;
    let mut container_autocomplete_data = AutocompleteData::default();
    for (name, container) in container_data.iter() {
        container_autocomplete_data
            .formatted
            .push(container.formatted_name.clone());
        container_autocomplete_data.unformatted.push(name.clone());
    }
    Ok(GameData {
        item_data,
        item_autocomplete_data,
        skin_unformatted_names,
        container_data,
        container_autocomplete_data,
    })
}
