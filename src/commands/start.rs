use crate::{
    bot::{Context, Error},
    util::{
        embed::{send_error_message_embed, send_author_not_registed_embed},
        string::currency_string
    },
};
use poise::{
    serenity_prelude as serenity,
    CreateReply
};
use serenity::CreateEmbed;

#[poise::command(
    description_localized("en-GB", "Start a case unboxing session"),
    prefix_command,
    slash_command
)]
pub async fn start(
    ctx: Context<'_>,
    #[description = "The amount of balance to use in the session"] amount: f64,
) -> Result<(), Error> {
    let id: u64 = ctx.author().id.into();
    let amount = (amount  * 100.0) as u64;
    if amount <= 0 {
        send_error_message_embed(&ctx, "Amount of balance **must** must be greater than zero").await?;
        return Ok(());
    }
    // Try and take balance from user to use in the session
    match sqlx::query_file!(
        "sql/try_deduct_balance.sql",
        id as i64,
        amount as i64
    )
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
    // Send embed
    let embed = CreateEmbed::default()
    .fields(vec![
        ("Remaining", currency_string(amount), true),
        ("Spent", currency_string(0), true),
        ("Return", currency_string(0), true),
    ]);
    ctx.send(CreateReply {
        embeds: vec![embed],
        reply: true,
        ..Default::default()
    }).await?;
    Ok(())
}
