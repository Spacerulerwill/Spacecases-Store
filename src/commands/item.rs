use crate::{
    bot::{Context, Error},
    data::types::{Item, Skin, Sticker},
    util::{
        embed::{item_quality_color, send_error_message_embed},
        string::{find_closest_match, normalize_item_name, to_title_case},
    },
};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{futures, futures::Stream, CreateEmbed, Timestamp};

fn skin_embed(skin: &Skin) -> Result<CreateEmbed, Error> {
    let found_in = &skin
        .containers_found_in
        .iter()
        .map(|line| format!("• {}", to_title_case(line)))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(CreateEmbed::default()
        .title(&skin.formatted_name)
        .description(format!("*{}*\n{}", &skin.flavor_text, &skin.description))
        .color(item_quality_color(&skin.quality)?)
        .image(&skin.image_urls[0])
        .field("Quality", to_title_case(&skin.quality), true)
        .field(
            "Float Range",
            format!("{} - {}", &skin.min_float, &skin.max_float),
            true,
        )
        .field("Found In", found_in, true)
        .timestamp(Timestamp::now()))
}

fn sticker_embed(sticker: &Sticker) -> Result<CreateEmbed, Error> {
    let found_in = &sticker
        .containers_found_in
        .iter()
        .map(|line| format!("• {}", to_title_case(line)))
        .collect::<Vec<String>>()
        .join("\n");

    Ok(CreateEmbed::default()
        .title(&sticker.formatted_name)
        .description(&sticker.description)
        .color(item_quality_color(&sticker.quality)?)
        .image(&sticker.image_urls[0])
        .field("Quality", to_title_case(&sticker.quality), true)
        .field("Found In", found_in, true)
        .timestamp(Timestamp::now()))
}

async fn autocomplete<'a>(ctx: Context<'a>, partial: &'a str) -> impl Stream<Item = String> + 'a {
    let mut autocomplete_result = Vec::new();
    let autocomplete_data = &ctx.data().game_data.item_autocomplete_data;
    for (formatted, unformatted) in autocomplete_data
        .formatted
        .iter()
        .zip(autocomplete_data.unformatted.iter())
    {
        if unformatted.starts_with(partial) {
            autocomplete_result.push(formatted.clone());
        }
    }
    return futures::stream::iter(autocomplete_result);
}

#[poise::command(
    description_localized("en-GB", "View an item"),
    prefix_command,
    track_edits,
    slash_command
)]
pub async fn item(
    ctx: Context<'_>,
    #[description = "The name of the item to view"]
    #[autocomplete = "autocomplete"]
    item: String,
) -> Result<(), Error> {
    let item_unformatted_name = normalize_item_name(&item);
    if let Some(item) = &ctx.data().game_data.item_data.get(&item_unformatted_name) {
        // Send an embed for the item
        let embed = match item {
            Item::Skin(skin) => skin_embed(&skin)?,
            Item::Sticker(sticker) => sticker_embed(&sticker)?,
        };
        ctx.send(CreateReply {
            embeds: vec![embed],
            reply: true,
            ..Default::default()
        })
        .await?;
    } else {
        // No item with this name - send error embed
        let closest = find_closest_match(
            &item,
            &ctx.data().game_data.item_autocomplete_data.unformatted,
            0.8,
        );
        match closest {
            Some(closest) => {
                send_error_message_embed(
                    &ctx,
                    format!(
                        "Item `{}` does not exist! Did you mean `{}`?",
                        item, closest
                    ),
                )
                .await?
            }
            None => {
                send_error_message_embed(&ctx, format!("Item **{}** does not exist!", item)).await?
            }
        }
    }
    Ok(())
}
