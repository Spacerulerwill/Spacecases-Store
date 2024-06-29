use crate::{
    bot::{Context, Error},
    util::embed::{send_author_not_registed_embed, send_yes_no_embed},
};
use poise::serenity_prelude as serenity;
use serenity::{Colour, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage};

#[poise::command(
    description_localized("en-GB", "Reset your SpaceCases progress"),
    prefix_command,
    slash_command
)]
pub async fn reset(ctx: Context<'_>) -> Result<(), Error> {
    let signed_id: i64 = u64::from(ctx.author().id) as i64;
    // Check they exist initially, early escape
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
        "Are you **sure** you want to reset your SpaceCases progress? This action is **irreversible!**",
        "Account reset **cancelled** as you did not respond in time",
    )
    .await?
    {
        Some(k) => k,
        None => return Ok(()),
    };
    if yes_chosen {
        // If yes chosen, reset user account stats
        let rows_affected = sqlx::query_file!("sql/reset_user.sql", signed_id)
            .execute(&ctx.data().database)
            .await?
            .rows_affected();
        if rows_affected == 0 {
            // If rows affected are zero, it must have been deleted after command invocation, before pressing yes
            let embed: CreateEmbed = CreateEmbed::default()
                .colour(Colour::DARK_RED)
                .description("**Failed** to reset account as it no longer exists");
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
        // Send embed of confirmation of account deletion
        let embed = CreateEmbed::default()
            .colour(Colour::DARK_GREEN)
            .description("Your account statics have been successfully reset");
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
            .description("Account reset cancelled as you failed to choose an option");
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
