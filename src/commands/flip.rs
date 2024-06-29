use crate::{
    bot::{Context, Error},
    util::{
        embed::{send_author_not_registed_embed, send_error_message_embed},
        images::{CT_LOGO, T_LOGO},
        string::currency_string,
    },
};
use poise::{
    serenity_prelude as serenity, {ChoiceParameter, CreateReply},
};
use serenity::{Colour, CreateEmbed, Timestamp};
use static_assertions::const_assert;

const REWARD_MULTIPLIER: f64 = 1.0;
const_assert!(REWARD_MULTIPLIER > 0.0);

#[derive(Debug, ChoiceParameter, PartialEq)]
pub enum CoinSideChoice {
    #[name = "Terrorist"]
    Terrorist,
    #[name = "Counter Terrorist"]
    CounterTerrorist,
}

#[poise::command(
    description_localized("en-GB", "Guess the right coin and win money"),
    prefix_command,
    slash_command
)]
pub async fn flip(
    ctx: Context<'_>,
    #[description = "Side of the coin to bet on"] side: CoinSideChoice,
    #[description = "The amount to bet on the coinflip"] bet: f64,
) -> Result<(), Error> {
    let bet = (bet * 100.0) as u64;
    if bet <= 0 {
        send_error_message_embed(&ctx, "You **must** bet an amount greater than zero").await?;
        return Ok(());
    }
    let signed_id: i64 = u64::from(ctx.author().id) as i64;
    let ct_chosen = side == CoinSideChoice::CounterTerrorist;
    let ct_won = rand::random::<f64>() > 0.5;
    let user_won = ct_chosen == ct_won;
    let mut win_amount = 0;
    let user_exists;
    let transaction_successful;

    // Make SQL requests based on whether user won or lost
    if user_won {
        win_amount = (bet as f64 * REWARD_MULTIPLIER) as u64;
        let result =
            sqlx::query_file!("sql/flip_win.sql", signed_id, bet as i64, win_amount as i64)
                .fetch_optional(&ctx.data().database)
                .await?;
        user_exists = !result.is_none();
        transaction_successful = user_exists && result.unwrap().transaction_successful.unwrap();
    } else {
        let result = sqlx::query_file!("sql/try_deduct_balance.sql", signed_id, bet as i64)
            .fetch_optional(&ctx.data().database)
            .await?;
        user_exists = !result.is_none();
        transaction_successful = user_exists && result.unwrap().transaction_successful.unwrap();
    }

    // Give message
    if user_exists {
        if transaction_successful {
            let embed_image;
            let embed_title;
            let embed_color;
            let embed_description;
            if ct_won {
                embed_image = CT_LOGO;
                embed_title = "Counter Terrorists Win!";
            } else {
                embed_image = T_LOGO;
                embed_title = "Terrorists Win!";
            }
            if user_won {
                embed_description = format!("You won **{}**", currency_string(win_amount));
                embed_color = Colour::DARK_GREEN;
            } else {
                embed_description = format!("You lost **{}**", currency_string(bet));
                embed_color = Colour::RED;
            }
            let embed = CreateEmbed::default()
                .image(embed_image)
                .title(embed_title)
                .colour(embed_color)
                .description(embed_description)
                .timestamp(Timestamp::now());
            ctx.send(CreateReply {
                embeds: vec![embed],
                reply: true,
                ..Default::default()
            })
            .await?;
        } else {
            send_error_message_embed(&ctx, "You do not have enough balance to bet this much!")
                .await?;
        }
    } else {
        send_author_not_registed_embed(&ctx).await?;
    }
    Ok(())
}
