use crate::{
    bot::{Context, Error},
    util::{
        embed::{get_user_avatar_url, send_author_not_registed_embed, send_error_message_embed},
        string::currency_string,
    },
};
use static_assertions::const_assert;
use {
    poise::{serenity_prelude as serenity, CreateReply},
    serenity::{Colour, CreateEmbed, CreateEmbedFooter},
};

pub const MENTION: &str = "</claim:1241073780978090177>";
/*
Claim streaks follow a simple arithmetic sequence where inital value is BASE_CLAIM
and the common difference is CLAIM_COMMON_DIFF, maxing out at MAX_CLAIM
*/
const BASE_CLAIM: u64 = 10000;
const CLAIM_COMMON_DIFF: u64 = 2500;
const MAX_CLAIM: u64 = 30000;
// Ensure claim streak values actually make sense
const_assert!(MAX_CLAIM >= BASE_CLAIM);
const_assert!(BASE_CLAIM % CLAIM_COMMON_DIFF == 0);
const_assert!(MAX_CLAIM % CLAIM_COMMON_DIFF == 0);

#[poise::command(
    description_localized("en-GB", "Claim your daily allowance"),
    prefix_command,
    slash_command
)]
pub async fn claim(ctx: Context<'_>) -> Result<(), Error> {
    let signed_id: i64 = u64::from(ctx.author().id) as i64;
    let result = sqlx::query_file!(
        "sql/claim.sql", 
        signed_id,
        BASE_CLAIM as i64,
        CLAIM_COMMON_DIFF as i64,
        MAX_CLAIM as i64
    )
        .fetch_optional(&ctx.data().database)
        .await?;

    match result {
        Some(row) => {
            if row.update_successful.unwrap() {
                // Send success embed
                let embed = CreateEmbed::new()
                    .title("You have successfully claimed your daily reward")
                    .description("Come back **tomorrow** to claim again!")
                    .footer(CreateEmbedFooter::new(
                        "Note: claim streaks reset after 1 day without claiming",
                    ))
                    .thumbnail(get_user_avatar_url(ctx.author()))
                    .fields(vec![
                        ("Amount", currency_string(row.change.unwrap() as u64), true),
                        ("New Balance", currency_string(row.balance as u64), true),
                        ("Streak 🔥", (row.claim_streak as u64).to_string(), true),
                    ])
                    .colour(Colour::DARK_GREEN);
                ctx.send(CreateReply {
                    reply: true,
                    embeds: vec![embed],
                    ..Default::default()
                })
                .await?;
            } else {
                send_error_message_embed(&ctx, "You can claim again tomorrow!").await?;
            }
        }
        None => send_author_not_registed_embed(&ctx).await?,
    }
    Ok(())
}
