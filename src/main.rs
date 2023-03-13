use std::env;


use songbird::input::Input;
use songbird::tracks::{PlayMode, TrackHandle, TrackState};
// This trait adds the `register_songbird` and `register_songbird_with` methods
// to the client builder below, making it easy to install this voice client.
// The voice client can be retrieved in any command using `songbird::get(ctx).await`.
use songbird::{SerenityInit};

// Import the `Context` to handle commands.
//use serenity::client::Context;

use serenity::{
    async_trait,
    client::{Client, EventHandler, Context},
    framework::{
        StandardFramework,
        standard::{
            Args, CommandResult,
            macros::{command, group},
        },
    },
    model::{channel::Message, gateway::Ready, gateway::Activity, gateway::ActivityType},
    prelude::GatewayIntents,
    Result as SerenityResult,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        let osrs = "Oldschool RuneScape: Getting Purples at Tombs of Amascut.";
        
        println!("{} is connected!", ready.user.name);
        
        context.set_activity(Activity::playing(osrs)).await;
        
    }

}

//Current list of commands
//When adding extra commands, you must add the command call to this list.
#[group]
#[commands(deafen, join, leave, mute, play_from_url, help, undeafen, unmute, stop, search_and_play, search_and_play_loop)]
struct General;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    //Configure the client with your Discord bot token in the environment.
    //EXPORT DISCORD_TOKEN='xxx' <- command that will export your token to env variable. 
    //Replace 'xxx' with your token
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c
                   .prefix("~"))
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
        let _ = client.start().await.map_err(|why| println!("Client ended: {:?}", why));
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

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "```Not in a voice channel```").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_deaf() {
        check_msg(msg.channel_id.say(&ctx.http, "```Already deafened```").await);
    } else {
        if let Err(e) = handler.deafen(true).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("```Failed: {:?}```", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "```Deafened```").await);
    }

    Ok(())
}

#[command]
#[aliases(comehere, summon)]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let channel_id = guild
        .voice_states.get(&msg.author.id)
        .and_then(|voice_state| voice_state.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            check_msg(msg.reply(ctx, "```Not in a voice channel```").await);

            return Ok(());
        }
    };
    check_msg(msg.channel_id.say(&ctx.http, "```Joined voice channel```").await);

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let _handler = manager.join(guild_id, connect_to).await;

    Ok(())
}

#[command]
#[aliases(goodbye, unjoin)]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("```Failed: {:?}```", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "```Left voice channel```").await);
    } else {
        check_msg(msg.reply(ctx, "```Not in a voice channel```").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            check_msg(msg.reply(ctx, "```Not in a voice channel```").await);

            return Ok(());
        },
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        check_msg(msg.channel_id.say(&ctx.http, "```Already muted```").await);
    } else {
        if let Err(e) = handler.mute(true).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("```Failed: {:?}```", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "```Now muted```").await);
    }

    Ok(())
}


#[command]
#[aliases(pfu, play)]
#[only_in(guilds)]
async fn play_from_url(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "```Must provide a URL to a video or audio```").await);

            return Ok(());
        },
    };

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say(&ctx.http, "```Must provide a valid URL```").await);

        return Ok(());
    }

    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::ytdl(&url).await {
            Ok(source) => source,
            Err(why) => {
                println!("```Err starting source: {:?}```", why);

                check_msg(msg.channel_id.say(&ctx.http, "```Error sourcing ffmpeg```").await);

                return Ok(());
            },
        };
        let title = source.metadata.title.clone().unwrap_or("Unknown".to_string());
        let tracktitle_to_be_displayed = format!("```Playing song: {title}```");
        
        
    
        handler.play_only_source(source);   
        
        
        check_msg(msg.channel_id.say(&ctx.http, tracktitle_to_be_displayed).await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "``````Not in a voice channel. If im playing audio contact the authorities!``````").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn undeafen(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.deafen(false).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("```Failed: {:?}```", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "```Undeafened```").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "```Not in a voice channel to undeafen in```").await);
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;
    
    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            check_msg(msg.channel_id.say(&ctx.http, format!("```Failed: {:?}```", e)).await);
        }

        check_msg(msg.channel_id.say(&ctx.http, "```Unmuted```").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "```Not in a voice channel to unmute in```").await);
    }

    Ok(())
}

#[command]
#[aliases(h)]
#[only_in(guilds)]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    check_msg(msg.channel_id.say(&ctx.http, 
        "```Hi! 
        To use Torkoal, 
        you first must be in a voice channel. From there, invite me to the channel by using the '~join' command.
        Here is a list of all the commands I currently accept: 
        [mute, unmute, deafen, undeafen, join, leave, play_from_url(pfu, play), search_and_play(sap), search_and_play_loop(sapl), stop and help]```").await);

    Ok(())
}

#[command]
#[aliases(sap)]
#[only_in(guilds)]
async fn search_and_play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    //collects command arg into a vector. with a space inbetween each word.
    //IE: ~search_and_play swimswim pier 34
    //prints out swimswim pier 34
    let arg_string = args.raw().collect::<Vec<&str>>().join(" ");
    //later used in ytdl_search() function to have a proper search query.
    
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
        
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match songbird::input::ytdl_search(&arg_string).await {
            Ok(source) => source,
            Err(why) => {
                println!("```Err starting source: {:?}```", why);

                check_msg(msg.channel_id.say(&ctx.http, "```Error sourcing ffmpeg```").await);

                return Ok(());
            },
        };
        
        let title = source.metadata.title.clone().unwrap_or("Unknown".to_string());
        let tracktitle_to_be_displayed = format!("```Playing song: {title}```");
        handler.play_only_source(source);
        
        
        check_msg(msg.channel_id.say(&ctx.http, tracktitle_to_be_displayed).await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "``````Not in a voice channel. If im playing audio contact the authorities!``````").await);
}
    
    
    Ok(())
}


#[command]
#[aliases(sapl)]
#[only_in(guilds)]
async fn search_and_play_loop(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    //collects command arg into a vector. with a space inbetween each word.
    //IE: ~search_and_play swimswim pier 34
    //prints out swimswim pier 34
    let arg_string = args.raw().collect::<Vec<&str>>().join(" ");
    //later used in ytdl_search() function to have a proper search query.
    
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
        .expect("Songbird Voice client placed in at initialisation.").clone();
        
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let source = match  songbird::input::restartable::Restartable::ytdl_search(&arg_string, true).await {
            Ok(source) => source,
            Err(why) => {
                println!("```Err starting source: {:?}```", why);

                check_msg(msg.channel_id.say(&ctx.http, "```Error sourcing ffmpeg```").await);

                return Ok(());
            },
        };
        let loopable_source_to_input_source = Input::from(source);
        let title = loopable_source_to_input_source.metadata.title.clone().unwrap_or("Unknown".to_string());
        let tracktitle_to_be_displayed = format!("```Playing song: {title}```");

        let loopable_trackhandle = handler.play_only_source(loopable_source_to_input_source);
        loopable_trackhandle.enable_loop().unwrap();
       
        check_msg(msg.channel_id.say(&ctx.http, tracktitle_to_be_displayed).await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "``````Not in a voice channel. If im playing audio contact the authorities!``````").await);
}
    Ok(())
}




#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    
    
    let guild = msg.guild(&ctx.cache).unwrap();
    let guild_id = guild.id;

    let manager = songbird::get(ctx).await
    .expect("Songbird Voice client placed in at initialisation.").clone();
        
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
    
       handler.stop();

        check_msg(msg.channel_id.say(&ctx.http, "```Stopping audio source```").await);
    } else {
        check_msg(msg.channel_id.say(&ctx.http, "```Not in a voice channel. If im playing audio contact the authorities!```").await);
}
    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
