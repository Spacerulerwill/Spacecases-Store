use std::time::Duration;

use crate::{
    bot::{Context, Error},
    data::types::Item,
    util::embed::{
        send_author_not_registed_embed, send_error_message_embed, send_success_message_embed,
    },
};
use poise::{serenity_prelude as serenity, CreateReply};
use rand::seq::SliceRandom;
use serenity::CreateEmbed;
use strsim::normalized_damerau_levenshtein;

const PRICE: u64 = 250;
const REWARD: u64 = 500;

#[poise::command(
    description_localized("en-GB", "Guess the skin and win money"),
    prefix_command,
    slash_command
)]
pub async fn skin_quiz(ctx: Context<'_>) -> Result<(), Error> {
    let signed_id = u64::from(ctx.author().id) as i64;
    // Try and reduce user balance
    match sqlx::query_file!("sql/try_deduct_balance.sql", signed_id, PRICE as i64)
        .fetch_optional(&ctx.data().database)
        .await?
    {
        Some(row) => {
            if !row.transaction_successful.unwrap() {
                send_error_message_embed(&ctx, "You do not have enough balance!").await?;
                return Ok(());
            }
        }
        None => {
            send_author_not_registed_embed(&ctx).await?;
            return Ok(());
        }
    }
    // Pick random skin and send embed
    let random_unformatted_skin_name = ctx
        .data()
        .game_data
        .skin_unformatted_names
        .choose(&mut rand::thread_rng())
        .unwrap();
    let item = ctx
        .data()
        .game_data
        .item_data
        .get(random_unformatted_skin_name)
        .unwrap();
    let skin = match item {
        Item::Skin(skin) => skin,
        _ => {
            return Err(format!(
                "Non-skin item found while selecting item for skin_quiz: {:?}",
                item
            )
            .into())
        }
    };
    let skin_name = skin.formatted_name.split("|").collect::<Vec<&str>>()[1]
        .trim()
        .to_lowercase();
    let embed = CreateEmbed::default()
    .title("Guess the skin!")
    .description("Reply with the name of the skin within 10 seconds!\nDo **not** include the wear or the weapon name!")
    .image(&skin.image_urls[0]);
    ctx.send(CreateReply {
        embeds: vec![embed],
        reply: true,
        ..Default::default()
    })
    .await?;
    // Await user response and respond based on whether they got it right or not
    let msg = ctx
        .author()
        .await_reply(&ctx)
        .timeout(Duration::from_secs(10))
        .await;
    match msg {
        Some(msg) => {
            let user_guess = &msg.content.trim().to_lowercase();
            if normalized_damerau_levenshtein(user_guess, &skin_name) > 0.85 {
                send_success_message_embed(&ctx, "You guessed **correctly!** You win **$5**")
                    .await?;
                sqlx::query_file!("sql/increase_balance.sql", (PRICE + REWARD) as i64, signed_id)
                    .execute(&ctx.data().database)
                    .await?;
            } else {
                send_error_message_embed(
                    &ctx,
                    format!(
                        "You guessed **incorrectly!** The correct answer was `{}`",
                        &skin_name
                    ),
                )
                .await?;
            }
        }
        None => {
            send_error_message_embed(
                &ctx,
                format!(
                    "You **failed** to respond within 10 seconds! The correct answer was `{}`",
                    &skin_name
                ),
            )
            .await?
        }
    }
    Ok(())
}
