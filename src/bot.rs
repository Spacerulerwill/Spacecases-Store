use crate::{
    commands,
    data::{fetch_game_data, types::GameData},
    util::embed::{get_welcome_embed, send_error_message_embed},
};
use log::{error, info, trace};
use poise::{serenity_prelude as serenity, FrameworkError};
use serenity::all::{ActivityData, ChannelId, CreateMessage, Guild};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{env, sync::Arc};

/*
A reference to this struct is passed to every slash command, it contains the
database and a reference to the game data needed for certain commands
*/
#[derive(Debug)]
pub struct Data {
    pub database: PgPool,
    pub game_data: GameData,
    pub user_count: Arc<AtomicUsize>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
pub type FrameworkContext<'a> = poise::FrameworkContext<'a, Data, Error>;

// A loop that cycles the bot status every couple seconds to a new message
async fn bot_status_cycle(ctx: serenity::Context, user_count: Arc<AtomicUsize>) {
    let mut status_msg = 0;
    loop {
        let message;
        match status_msg {
            0 => message = String::from("Use /help"),
            1 => {
                message = format!(
                    "{} Users | {} Guilds",
                    user_count.load(Ordering::SeqCst),
                    ctx.cache.guilds().len()
                );
            }
            _ => panic!("This should never happen! Tell a programmer"),
        }
        ctx.set_activity(Some(ActivityData::playing(message)));
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        status_msg = (status_msg + 1) % 2;
    }
}

// Send a welcome message to the discord server our bot just joined
async fn send_welcome_message(ctx: &serenity::Context, guild: &Guild) -> Result<(), Error> {
    async fn send_to_text_channel(
        ctx: &serenity::Context,
        channel_id: ChannelId,
    ) -> Result<(), Error> {
        channel_id
            .send_message(
                ctx,
                CreateMessage::default().add_embed(get_welcome_embed(ctx)),
            )
            .await?;
        Ok(())
    }
    if let Some(system_channel_id) = guild.system_channel_id {
        // Try and send message to  system channel
        if let Ok(_) = send_to_text_channel(ctx, system_channel_id).await {
            return Ok(());
        }
    } else {
        for (channel_id, _) in guild.channels.iter() {
            // Try and send message to any text channel in the server
            if let Ok(_) = send_to_text_channel(ctx, *channel_id).await {
                return Ok(());
            }
        }
    }
    // send to server owner if we weren't able to send to any text channels
    if let Ok(dm_channel) = guild.owner_id.create_dm_channel(ctx).await {
        let _ = dm_channel
            .send_message(
                ctx,
                CreateMessage::default().add_embed(get_welcome_embed(ctx)),
            )
            .await;
    }
    // otherwise we give up
    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // Inner function so we can propogate errors
    async fn inner(error: poise::FrameworkError<'_, Data, Error>) -> Result<(), Error> {
        match error {
            FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
            FrameworkError::Command { error, ctx, .. } => {
                error!("Error in command `{}`: {:?}", ctx.command().name, error);
                send_error_message_embed(
                    &ctx,
                    &format!("An error has occured while executing command `{}`, it has been reported to the development team", 
                    &ctx.command().name))
                    .await?;
            }
            error => {
                if let Err(e) = poise::builtins::on_error(error).await {
                    error!("Error while handling error: {}", e)
                }
            }
        }
        Ok(())
    }
    // If we encounter an error while handling an error (e.g. error message embed fails to send)
    if let Err(err) = inner(error).await {
        error!("Error while handling error: {}", err);
    }
}

async fn pre_command(ctx: Context<'_>) {
    trace!("Executing command {}!", ctx.command().qualified_name);
}

async fn post_command(ctx: Context<'_>) {
    trace!("Executed command {}!", ctx.command().qualified_name);
}

async fn on_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: FrameworkContext<'_>,
    data: &Data,
) -> Result<(), Error> {
    trace!(
        "Got an event in event handler: {:?}",
        event.snake_case_name()
    );
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            // Bot status loop
            info!("Logged in as {}", data_about_bot.user.name);
            tokio::task::spawn(bot_status_cycle(ctx.clone(), data.user_count.clone()));
        }
        serenity::FullEvent::GuildCreate { guild, is_new } => {
            // Send welcome message
            if is_new.unwrap_or(false) {
                send_welcome_message(ctx, guild).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

// Connect to the database using the DATABASE_URL env variable and run migrations
async fn database_init() -> PgPool {
    info!("Establishing database connection");
    let database = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL environment variable not provided"))
        .await
        .expect("Couldn't connect to database");
    sqlx::migrate!("sql/migrations/")
        .run(&database)
        .await
        .expect("Couldn't run database migrations");
    database
}

// Setup poise and run the bot
async fn poise_init(database: PgPool) {
    // Setup bot configurations - callbacks, commands etc
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::balance::balance(),
            commands::claim::claim(),
            commands::item::item(),
            commands::register::register(),
            commands::skin_quiz::skin_quiz(),
            commands::flip::flip(),
            commands::reset::reset(),
            commands::delete::delete(),
            commands::transfer::transfer(),
            commands::start::start()
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("cs ".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                std::time::Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        pre_command: |ctx| Box::pin(pre_command(ctx)),
        post_command: |ctx| Box::pin(post_command(ctx)),
        skip_checks_for_owners: false,
        event_handler: |ctx, event, framework, data| {
            Box::pin(async move {
                on_event(ctx, event, framework, data).await?;
                Ok(())
            })
        },
        ..Default::default()
    };

    /*
    Create bot framework - will register guild to a server if env variable TEST_GUILD is provided
    otherwise it will register the commands globally
    */
    let framework = poise::Framework::builder()
        .setup(move |ctx, _, framework| {
            Box::pin(async move {
                // Register slash commands
                info!("Registering slash commands");
                if let Ok(guild_id_string) = env::var("TEST_GUILD") {
                    let guild_id: u64 = guild_id_string
                        .parse()
                        .expect("TEST_GUILD must be a unsigned 64 bit integer");
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        guild_id.into(),
                    )
                    .await?;
                } else {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }
                // Obtain number of current users
                info!("Counting initial users");
                let row_count = sqlx::query_scalar!(r#"SELECT COUNT(*) FROM "user";"#,)
                    .fetch_one(&database)
                    .await?
                    .unwrap() as usize;
                let user_count = Arc::new(AtomicUsize::new(row_count));
                // Fetch the CS2 game data required for the bot to run from my super cool CS2 API
                info!("Fetching CS2 API data");
                let game_data = fetch_game_data()
                    .await
                    .expect("Failed to unmarshall item data from CS2-API: {err:?}");
                Ok(Data {
                    database,
                    game_data,
                    user_count,
                })
            })
        })
        .options(options)
        .build();
    // Start bot with token and appropriate intents
    info!("Starting bot");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN environment variable not found");
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Failed to create client");
    client.start().await.expect("Failed to start client");
}

pub async fn start() {
    let database = database_init().await;
    poise_init(database).await;
}
