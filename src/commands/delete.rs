use crate::{
    bot::{Context, Error},
    commands,
    util::embed::{send_author_not_registed_embed, send_yes_no_embed},
};
use std::sync::atomic::Ordering;
use {
    poise::serenity_prelude as serenity,
    serenity::{Colour, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage},
};

#[poise::command(
    description_localized("en-GB", "Delete your SpaceCases progress"),
    prefix_command,
    slash_command
)]
pub async fn delete(ctx: Context<'_>) -> Result<(), Error> {
    // Initial check to ensure they exist
    let signed_id: i64 = u64::from(ctx.author().id) as i64;
    let exists = sqlx::query_file_scalar!("sql/user_exist.sql", signed_id)
        .fetch_one(&ctx.data().database)
        .await?
        .unwrap_or(false);
    if !exists {
        send_author_not_registed_embed(&ctx).await?;
        return Ok(());
    }
    // If they exists, continue wiwth process - ask a yes or no option box
    let (yes_chosen, interact) = match send_yes_no_embed(
        &ctx,
        "Are you **sure** you want to delete your SpaceCases account? All progress will be **permanently** lost!", 
        "Deletion of account **cancelled** as you did not respond in time").await? {
        Some(k) => k,
        None => return Ok(()),
    };
    if yes_chosen {
        // If yes chosen, delete account and get rows affected
        let rows_affected = sqlx::query_file!("sql/delete_user.sql", signed_id)
            .execute(&ctx.data().database)
            .await?
            .rows_affected();
        if rows_affected == 0 {
            // If rows affected are zero, it must have been deleted after command invocation, before pressing yes
            let embed: CreateEmbed = CreateEmbed::default()
                .colour(Colour::DARK_RED)
                .description("**Failed** to delete account as it no longer exists");
            // Update original message
            interact
                .create_response(
                    ctx,
                    CreateInteractionResponse::UpdateMessage(
                        CreateInteractionResponseMessage::default()
                            .embed(embed)
                            .components(vec![]),
                    ),
                )
                .await?;
            return Ok(());
        }
        if rows_affected == 1 {
            // If one row deleted decrement global user count
            ctx.data().user_count.fetch_sub(1, Ordering::SeqCst);
        }
        // Send embed of confirmation of account deletion
        let embed = CreateEmbed::default()
            .colour(Colour::DARK_GREEN)
            .description(&format!(
                "Your account has been **deleted**. You can use {} at any time to create a new one",
                commands::register::MENTION
            ));
        interact
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .embed(embed)
                        .components(vec![]),
                ),
            )
            .await?;
    } else {
        // If no chosen, edit message and do nothing
        let embed = CreateEmbed::default()
            .colour(Colour::RED)
            .description("Account deletion **cancelled**");
        interact
            .create_response(
                ctx,
                CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::default()
                        .embed(embed)
                        .components(vec![]),
                ),
            )
            .await?;
    }
    Ok(())
}
