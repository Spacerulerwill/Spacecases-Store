use crate::{
    bot::{Context, Error},
    util::{
        embed::{
            send_author_not_registed_embed, send_error_message_embed, send_member_not_registered,
            send_success_message_embed,
        },
        string::currency_string,
    },
};
use {poise::serenity_prelude as serenity, serenity::Member};
#[poise::command(
    description_localized("en-GB", "Transfer balance to another user"),
    prefix_command,
    slash_command
)]
pub async fn transfer(
    ctx: Context<'_>,
    #[description = "User to transfer balance too"] member: Member,
    #[description = "Amount of balance to transfer"] amount: f64,
) -> Result<(), Error> {
    let amount = (amount * 100.0) as u64;
    if amount <= 0 {
        send_error_message_embed(&ctx, "You **must** transfer an amount greater than zero").await?;
        return Ok(());
    }
    let sender_id = u64::from(ctx.author().id);
    let recipient_id = u64::from(member.user.id);
    if sender_id == recipient_id {
        send_error_message_embed(&ctx, "You cannot transfer balance to yourself!").await?;
        return Ok(());
    }
    let mut tx = ctx.data().database.begin().await?;
    // Try and reduce balance from sender
    match sqlx::query_file!(
        "sql/try_deduct_balance.sql",
        sender_id as i64,
        amount as i64
    )
    .fetch_optional(&mut *tx)
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
    // Try and increase balance on recipient
    let rows_affected = sqlx::query_file!(
        "sql/increase_balance.sql",
        amount as i64,
        recipient_id as i64
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if rows_affected == 0 {
        send_member_not_registered(&ctx, &member).await?;
        return Ok(());
    } else {
        send_success_message_embed(
            &ctx,
            &format!(
                "Successfully transferred **{}** to **{}**",
                &currency_string(amount),
                &member.display_name()
            ),
        )
        .await?
    }
    tx.commit().await?;
    Ok(())
}
