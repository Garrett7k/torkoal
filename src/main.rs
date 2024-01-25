use serenity::builder::CreateEmbedFooter;
use songbird::tracks::PlayMode;
use songbird::{driver::Driver, id::GuildId, input::Input, tracks::TrackQueue};

use std::env;
use std::thread::sleep;
use std::time::{Duration, Instant};

use std::fs;
use std::fs::File;
use std::path::Path;

// This trait adds the `register_songbird` and `register_songbird_with` methods
// to the client builder below, making it easy to install this voice client.
// The voice client can be retrieved in any command using `songbird::get(ctx).await`.
use songbird::{SerenityInit, TrackEvent};

// Import the `Context` to handle commands.
//use serenity::client::Context;

use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args, CommandResult,
        },
        StandardFramework,
    },
    model::{channel::Message, gateway::Activity, gateway::Ready, prelude::EmojiIdentifier},
    prelude::GatewayIntents,
    Result as SerenityResult,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        let osrs =
            "Oldschool RuneScape: Getting close with Verzik. Maybe she loves me? ~Help for help!";
        let user = ready.user;

        if let Ok(guilds) = user.guilds(&context.http).await {
            for (_, guild) in guilds.into_iter().enumerate() {
                println!("{} is connected to: {}", user.name, guild.name);
            }
        }
        context.set_activity(Activity::playing(osrs)).await;
    }
}

//Current list of commands
//When adding extra commands, you must add the command call to this list.
#[group]
#[commands(
    deafen,
    join,
    leave,
    mute,
    help,
    undeafen,
    unmute,
    stop,
    search_and_play,
    aliases,
    skip
)]
struct General;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    //Configure the client with your Discord bot token in the environment.
    //EXPORT DISCORD_TOKEN='xxx' <- command that will export your token to env variable.
    //Replace 'xxx' with your token
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| {
            c.prefixes(vec!["!", ">", "~", ".", ",", "`", "-"])
                .case_insensitivity(true)
        })
        .group(&GENERAL_GROUP);

    //bitwise operand to provide an instance of the GatewayIntents struct that has both the functionality of non_privileged and MESSAGE_CONTENT gateway intents.
    //non_privileged:   1 1 0 0 1 0 1
    //MESSAGE_CONTENT:   0 0 0 1 0 0 0
    //| | | | | | | |
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    //intents: 1 1 0 1 1 0 1
    //1101101 is the binary value of the intents variable that has both non_privileged and MESSAGE_CONTENT gateway intents.

    //Initialize bot client with preset token, intents, handler, framework
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .expect("Err creating client");

    //Spawns the Discord Client asynchronously - allowing for the client to stay connected to the Discord API while recieving and responding to chat commands.
    //If an error occurs, this will return with the error.
    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| println!("Client ended: {:?}", why));
    });

    //awaits a termination ctrl-c command in terminal that is running the bot - when recieved, the client will end and the bot will disconnect from all guilds.
    tokio::signal::ctrl_c().await.unwrap();
    println!("Received Ctrl-C, shutting down.");
}

#[command]
#[only_in(guilds)]
async fn deafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "```Not in a voice channel```").await);

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;
    let msg_author = &msg.author.name;
    let deaf = format!("```{msg_author} deafened.```");

    if handler.is_deaf() {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "```Already deafened```")
                .await,
        );
    } else {
        if let Err(e) = handler.deafen(true).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("```Failed: {e:?}```"))
                    .await,
            );
        }

        check_msg(msg.channel_id.say(&ctx.http, deaf).await);
    }
    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[aliases(comehere, summon, sum, j, come, ch)]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states
        .get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "```Not in a voice channel```").await);
            msg_clean_up(ctx, msg).await;
            return Ok(());
        }
    };
    let msg_author = &msg.author.name;
    let summon = format!("```{msg_author} summoned.```");
    check_msg(msg.channel_id.say(&ctx.http, summon).await);

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let _handler = manager.join(guild_id, connect_to).await;
    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[aliases(goodbye, unjoin, l, gb, uj)]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("```Failed: {e:?}```"))
                    .await,
            );
        }
        let msg_author = &msg.author.name;
        let unsummon = format!("```{msg_author} unsummoned.```");

        check_msg(msg.channel_id.say(&ctx.http, unsummon).await);
    } else {
        check_msg(msg.reply(ctx, "```Not in a voice channel```").await);
    }
    let osrs =
        "Oldschool RuneScape: Getting close with Verzik. Maybe she loves me? ~Help for help!";
    ctx.set_activity(Activity::playing(osrs)).await;
    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "```Not in a voice channel```").await);

            return Ok(());
        }
    };
    let msg_author = &msg.author.name;
    let muted = format!("```{msg_author} muted.```");

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        check_msg(msg.channel_id.say(&ctx.http, "```Already muted```").await);
    } else {
        if let Err(e) = handler.mute(true).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("```Failed: {e:?}```"))
                    .await,
            );
        }

        check_msg(msg.channel_id.say(&ctx.http, muted).await);
    }
    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(false).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("```Failed: {e:?}```"))
                    .await,
            );
        }
        let msg_author = &msg.author.name;
        let undeaf = format!("```{msg_author} Undeafened.```");

        check_msg(msg.channel_id.say(&ctx.http, undeaf).await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "```Not in a voice channel to undeafen in```")
                .await,
        );
    }
    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("```Failed: {e:?}```"))
                    .await,
            );
        }
        let msg_author = &msg.author.name;
        let unmute = format!("```{msg_author} Unmuted.```");

        check_msg(msg.channel_id.say(&ctx.http, unmute).await);
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, "```Not in a voice channel to unmute in```")
                .await,
        );
    }
    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[aliases(sap, p, play, pfu, listen, find, audio, search, map)]
#[only_in(guilds)]
async fn search_and_play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    //collects command arg into a vector. with a space inbetween each word.
    //IE: ~commamd/alias arg arg arg
    //prints out arg arg arg
    let arg_string = args.raw().collect::<Vec<&str>>().join(" ");
    //later used in ytdl_search() function to have a proper search query.

    //set a time to calc initialisation of command.
    let now = Instant::now();

    //returns guild information from the sent message of user requesting this command
    let guild = msg.guild(&ctx.cache).unwrap();
    //denote the specific guild ID information
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let queue = handler.queue().clone();

        let source = match songbird::input::ytdl_search(&arg_string).await {
            Ok(source) => source,
            Err(why) => {
                println!("```Err starting source (is this where it fails?): {why:?}```");
                let msg_author = &msg.author.name;
                let ffmpeg_error = format!("```{msg_author} - FFMPEG Error! {why}. Check command Arguments are correct.```");
                check_msg(msg.channel_id.say(&ctx.http, ffmpeg_error).await);
                msg_clean_up(ctx, msg).await;
                return Ok(());
            }
        };

        let title = source
            .metadata
            .title
            .clone()
            .unwrap_or("Unknown".to_string());

        let source_artist = source
            .metadata
            .artist
            .clone()
            .unwrap_or("Unknown".to_string());

        let thumbnail = source.metadata.thumbnail.clone().unwrap();
        let link = "https://images-wixmp-ed30a86b8c4ca887773594c2.wixmp.com/f/7fbd888d-4a1d-41f7-ab72-404af6f4eec7/d3kmeku-93edd860-02ec-4d2e-b2a6-92aae3cc5b2a.png?token=eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJ1cm46YXBwOjdlMGQxODg5ODIyNjQzNzNhNWYwZDQxNWVhMGQyNmUwIiwic3ViIjoidXJuOmFwcDo3ZTBkMTg4OTgyMjY0MzczYTVmMGQ0MTVlYTBkMjZlMCIsImF1ZCI6WyJ1cm46c2VydmljZTpmaWxlLmRvd25sb2FkIl0sIm9iaiI6W1t7InBhdGgiOiIvZi83ZmJkODg4ZC00YTFkLTQxZjctYWI3Mi00MDRhZjZmNGVlYzcvZDNrbWVrdS05M2VkZDg2MC0wMmVjLTRkMmUtYjJhNi05MmFhZTNjYzViMmEucG5nIn1dXX0.grsn79H7WZObDpRz6cYgXyA9fyucGzE_Y4VgQkXCRHQ";

        let msg_author = &msg.author.name;
        let dur = source.metadata.duration.clone().unwrap().as_secs() / 60;
        let timeframe = now.elapsed().as_millis();

        //this is current playing song.

        if let Some(current) = queue.clone().current() {
            let current = current;
            let current_title = current
                .metadata()
                .title
                .clone()
                .unwrap_or("Unknown".to_string());
            let current_time = current.get_info().await.unwrap();
            let _etocs = current_time.position.as_secs();
            let tpt = current_time.play_time;
            let _test = current_time.playing;

            check_msg(
                msg.channel_id
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title(format!("Queued:"))
                                .description(title.clone())
                                .thumbnail(link)
                                .image(thumbnail.clone())
                                .fields(vec![
                                    (
                                        format!("Channel name:  {source_artist} "),
                                        format!("Currently playing: {}", current_title.clone()),
                                        true,
                                    ),
                                    (
                                        format!("Requested by: {msg_author}"),
                                        format!("Current time in track: {tpt:?}"),
                                        true,
                                    ),
                                ])
                        })
                    })
                    .await,
            );
        } else {
            let msg_author = &msg.author.name;
            check_msg(msg
            .channel_id
            .send_message(&ctx.http,|m|
            m
            .embed(|e| e.title(format!("Now Playing (No other songs in queue):")).description(title.clone()).thumbnail(link).image(thumbnail.clone()).fields(vec![
            (format!("Channel name:  {source_artist} "), format!("Command initialized, acquired search perimeters, audio executed in {timeframe} ms"), true),
            (format!("Requested by: {msg_author}"), format!("Track Duration: {dur:?} Minutes"), true),
        ]))).await);
        };

        let _song = handler.enqueue_source(source);
    } else {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    "```Not in a voice channel. If im playing audio contact the authorities!```",
                )
                .await,
        );
    }

    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let qh = handler.queue().clone();
        let qsh = qh.stop();

        let msg_author = &msg.author.name;
        let msg_display = format!("``` {msg_author} Stopped current audio source.```");

        check_msg(msg.channel_id.say(&ctx.http, msg_display).await);
    } else {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    "```Not in a voice channel. If im playing audio contact the authorities!```",
                )
                .await,
        );
    }
    let osrs =
        "Oldschool RuneScape: Getting close with Verzik. Maybe she loves me? ~Help for help!";
    ctx.set_activity(Activity::playing(osrs)).await;
    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[aliases(h)]
#[only_in(guilds)]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http,
        "
        ```
        Hi! 
        To use Torkoal, 
        you first must be in a voice channel. From there, invite me to the channel by using the '~join' command.
        Here is a list of all the commands I currently accept: 

        [mute, unmute, deafen, undeafen, join, leave, play_from_url, search_and_play, search_and_play_loop, stop, remind and help].
        
        Use the ~Aliases command for command aliases.
        ```").await);

    msg_clean_up(ctx, msg).await;
    Ok(())
}

#[command]
#[aliases(a)]
#[only_in(guilds)]
async fn aliases(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(
        msg.channel_id
            .say(
                &ctx.http,
                "
        ```
        Join -  comehere, summon, sum, j, come, ch
        Help - h
        Play_from_url - p, play, pfu
        Search_and_play - sap
        Search_and_play_loop - sapl
        Aliases - a
        Stop - s
        Leave - goodbye, unjoin, l, gb, uj
        Remind - r
        ```",
            )
            .await,
    );

    msg_clean_up(ctx, msg).await;

    Ok(())
}

async fn msg_clean_up(ctx: &Context, msg: &Message) {
    msg.delete(&ctx.http).await.unwrap();
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();

        let _ = queue.skip();

        let msg_author = &msg.author.name;
        let msg_display = format!("``` {msg_author} Skipped current audio source.```");

        check_msg(msg.channel_id.say(&ctx.http, msg_display).await);
    } else {
        check_msg(
            msg.channel_id
                .say(
                    &ctx.http,
                    "```Not in a voice channel. If im playing audio contact the authorities!```",
                )
                .await,
        );
    }
    msg_clean_up(ctx, msg).await;
    Ok(())
}
