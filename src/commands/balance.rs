use crate::{
    bot::{Context, Error},
    util,
};
use util::{
    embed::{
        send_author_not_registed_embed, send_member_not_registered, send_success_message_embed,
    },
    string::currency_string,
};
use {poise::serenity_prelude as serenity, serenity::Member};

#[poise::command(
    description_localized("en-GB", "Check your SpaceCases bank balance"),
    prefix_command,
    slash_command,
    aliases("bal")
)]
pub async fn balance(
    ctx: Context<'_>,
    #[description = "User to check balance of"] member: Option<Member>,
) -> Result<(), Error> {
    let member_id = match member {
        Some(ref member) => member.user.id,
        None => ctx.author().id,
    };
    // Get user balance
    let signed_id = u64::from(member_id) as i64;
    let balance = sqlx::query_file_scalar!("sql/balance.sql", signed_id)
        .fetch_optional(&ctx.data().database)
        .await?;
    // Display message
    match balance {
        // The user exists while trying to get balance - show them balance
        Some(signed_balance) => {
            let msg = match member {
                Some(member) => format!(
                    "**{}'s** balance is: **{}**",
                    &member.display_name(),
                    &currency_string(signed_balance as u64)
                ),
                None => format!(
                    "Your balance is: **{}**",
                    &currency_string(signed_balance as u64)
                ),
            };
            send_success_message_embed(&ctx, &msg).await?;
        }
        // No user found - they aren't registered yet
        None => {
            match member {
                Some(ref member) => send_member_not_registered(&ctx, member).await?,
                None => send_author_not_registed_embed(&ctx).await?,
            };
        }
    }
    Ok(())
}
