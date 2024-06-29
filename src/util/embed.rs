use crate::{
    bot::{Context, Error},
    commands,
};
use poise::{serenity_prelude as serenity, CreateReply};
use serenity::{
    ButtonStyle, Colour, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed, Member,
    User,
};
use std::time::Duration;

pub fn get_welcome_embed(ctx: &serenity::Context) -> CreateEmbed {
    CreateEmbed::default()
        .description(format!(
            "Hello! My name is **SpaceCases**
    
    I am a CS2 case unboxing, trading and economy bot. With me, you can:
    • Unbox your dream skins
    • Play minigames to earn balance
    • Trade items with your friends
    • Compete against your friends on a global leaderboard
    
    Enjoy! - [Spacerulerwill](https://github.com/Spacerulerwill)"
        ))
        .thumbnail(match ctx.cache.current_user().avatar_url() {
            Some(url) => url,
            None => ctx.cache.current_user().default_avatar_url(),
        })
}

/// Get embed color for an item from its item quality
pub fn item_quality_color(quality: &str) -> Result<Colour, Error> {
    match quality {
        "consumer grade" | "base grade" => Ok(Colour(0xb0c3d9)),
        "industrial grade" => Ok(Colour(0x5e98d9)),
        "mil-spec" | "high grade" => Ok(Colour(0x4b69ff)),
        "restricted" | "remarkable" => Ok(Colour(0x8847ff)),
        "classified" | "exotic" => Ok(Colour(0xd32ee6)),
        "covert" | "extraordinary" => Ok(Colour(0xeb4b4b)),
        "contraband" => Ok(Colour(0xffae39)),
        _ => Err(format!("Invalid item quality: {}", quality).into()),
    }
}

pub async fn send_text_embed(
    ctx: &Context<'_>,
    message: impl Into<String>,
    colour: Colour,
) -> Result<(), Error> {
    let embed = CreateEmbed::default().color(colour).description(message);
    ctx.send(CreateReply {
        embeds: vec![embed],
        reply: true,
        ..Default::default()
    })
    .await?;
    Ok(())
}

pub async fn send_success_message_embed(
    ctx: &Context<'_>,
    message: impl Into<String>,
) -> Result<(), Error> {
    send_text_embed(ctx, message, Colour::DARK_GREEN).await?;
    Ok(())
}

pub async fn send_error_message_embed(
    ctx: &Context<'_>,
    message: impl Into<String>,
) -> Result<(), Error> {
    send_text_embed(ctx, message, Colour::RED).await?;
    Ok(())
}

pub async fn send_author_not_registed_embed(ctx: &Context<'_>) -> Result<(), Error> {
    send_error_message_embed(
        ctx,
        &format!(
            "You are **not** registered! Use {} to register",
            commands::register::MENTION
        ),
    )
    .await?;
    Ok(())
}

pub async fn send_member_not_registered(ctx: &Context<'_>, member: &Member) -> Result<(), Error> {
    send_error_message_embed(
        ctx,
        &format!("**{}** is **not** registered!", member.display_name()),
    )
    .await?;
    Ok(())
}

pub async fn send_yes_no_embed(
    ctx: &Context<'_>,
    message: impl Into<String>,
    no_response_message: impl Into<String>,
) -> Result<Option<(bool, ComponentInteraction)>, Error> {
    // Embed with 2 buttons, yes and no
    let yes_button = CreateButton::new("y")
        .style(ButtonStyle::Success)
        .label("Yes");
    let no_button = CreateButton::new("n")
        .style(ButtonStyle::Danger)
        .label("No");
    let action_row = CreateActionRow::Buttons(vec![yes_button, no_button]);
    let embed = serenity::CreateEmbed::default().description(message);
    // Sendmessage
    let message = ctx
        .send(CreateReply {
            embeds: vec![embed],
            components: Some(vec![action_row]),
            reply: true,
            ..Default::default()
        })
        .await?;
    // Await interaction
    let interact = match message
        .message()
        .await?
        .await_component_interactions(&ctx)
        .timeout(Duration::from_secs(60))
        .await
    {
        Some(x) => x,
        None => {
            // Edit to message to show timeout message
            let edit_embed = serenity::CreateEmbed::default()
                .description(no_response_message)
                .colour(Colour::RED);
            message
                .edit(
                    *ctx,
                    CreateReply::default().embed(edit_embed).components(vec![]),
                )
                .await?;
            return Ok(None);
        }
    };
    // Modify original message and return correct boolean depending on button clicked
    match interact.data.custom_id.as_str() {
        "y" => return Ok(Some((true, interact))),
        _ => return Ok(Some((false, interact))),
    };
}

pub fn get_user_avatar_url(user: &User) -> String {
    match user.avatar_url() {
        Some(url) => url,
        None => user.default_avatar_url(),
    }
}
