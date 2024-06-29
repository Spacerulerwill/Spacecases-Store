use crate::{
    bot::{Context, Error},
    commands,
    util::embed::{send_error_message_embed, send_success_message_embed},
};
use std::sync::atomic::Ordering;

pub const MENTION: &str = "</register:1241073780978090179>";

#[poise::command(
    description_localized("en-GB", "Register a SpaceCases bank account"),
    prefix_command,
    slash_command
)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    // Try insert user into database
    let id: i64 = ctx.author().id.into();
    let rows_affected = sqlx::query_file!("sql/register.sql", id)
        .execute(&ctx.data().database)
        .await?
        .rows_affected();
    // Send message depending on whether the insert happened
    // If it didn't happen they are already registered
    if rows_affected == 0 {
        send_error_message_embed(&ctx, "You are **already** registered!").await?;
    } else {
        ctx.data().user_count.fetch_add(1, Ordering::SeqCst);
        send_success_message_embed(
            &ctx,
            &format!(
                "Registered! Use {} to claim your daily balance",
                commands::claim::MENTION
            ),
        )
        .await?;
    }
    Ok(())
}
