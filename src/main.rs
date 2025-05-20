use scryfall::Card;
use std::env;
use std::time::Instant;

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
    model::{channel::Message, gateway::Activity, gateway::Ready},
    prelude::GatewayIntents,
    Result as SerenityResult,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        let osrs = "Pondering Scryfall..";
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
#[commands(card, doublecard, blackwhitelands, randomsearch)]
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
            c.prefixes(vec!["!", ">", "~", ".", ",", "`", "-", "["])
                .case_insensitivity(true)
        })
        .group(&GENERAL_GROUP);

    //bitwise operand to provide an instance of the GatewayIntents struct that has both the functionality of non_privileged and MESSAGE_CONTENT gateway intents.
    //non_privileged:   1 1 0 0 1 0 1
    //MESSAGE_CONTENT:   0 0 0 1 0 0 0
    //| | | | | | | |
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::DIRECT_MESSAGES;
    //intents: 1 1 0 1 1 0 1
    //1101101 is the binary value of the intents variable that has both non_privileged and MESSAGE_CONTENT gateway intents.

    //Initialize bot client with preset token, intents, handler, framework
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        // .register_songbird()
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
#[aliases(scry, s)]
#[only_in(guilds)]
async fn card(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg_clean_up(ctx, msg).await;

    let card_name = args.raw().collect::<Vec<&str>>().join(" ");
    let now = Instant::now();

    let card = match Card::named_fuzzy(card_name.as_str()).await {
        Ok(card) => card,
        Err(e) => panic!(
            "{}",
            format!(
                "{:?}, {:?}",
                e.to_string(),
                check_msg(msg.channel_id.say(&ctx.http, e.to_string()).await),
            )
        ),
    };

    let card_legal_m = card.legalities.modern;
    let card_legal_p = card.legalities.pauper;
    let card_legal_c = card.legalities.commander;
    let card_set = card.set_name;

    let _card_price = match card.prices.usd {
        Some(card_price) => format!("{:?}", check_msg(
               msg.channel_id
                    .say(&ctx.http, format!("Card Price (USD): ${card_price}"))
                    .await,
            )),
        None => format!(
            "{:?}",
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Unable to find Card Price (USD)"))
                    .await
            )
        ),
    };

    let card_image = match card.image_uris {
    	Some(card_image) => card_image.png,
    	None => panic!("{}", format!(
                "{:?}, {:?}",
                check_msg(msg.channel_id.say(&ctx.http, format!("Unable to find Card Image (Possibly a double faced card). Here is Scryfall link:")).await),
       			 check_msg(msg.channel_id.say(&ctx.http, card.scryfall_uri).await)
            ))
    };

    check_msg(msg.channel_id.say(&ctx.http, card_image.unwrap()).await);
    let timeframe = now.elapsed().as_millis();
    check_msg(msg.channel_id.say(&ctx.http, format!("Fetch time in ms: {timeframe}\nModern: {card_legal_m:?}\nPauper: {card_legal_p:?}\nCommander: {card_legal_c:?}\nPrinting: {card_set}")).await);

    Ok(())
}

#[command]
#[aliases(sdf)]
#[only_in(guilds)]
async fn doublecard(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    msg_clean_up(ctx, msg).await;

    let card_name = args.raw().collect::<Vec<&str>>().join(" ");
    let now = Instant::now();

    let card = match Card::named_fuzzy(card_name.as_str()).await {
        Ok(card) => card,
        Err(e) => panic!(
            "{}",
            format!(
                "{:?}",
                check_msg(msg.channel_id.say(&ctx.http, e.to_string()).await)
            )
        ),
    };

    let card_price = card.prices.usd;
    let card_face = card.card_faces;
    let card_legal_m = card.legalities.modern;
    let card_legal_p = card.legalities.pauper;
    let card_legal_c = card.legalities.commander;

    if let Some((cf, cp)) = card_face.zip(card_price) {
        let fs = cf[0].clone();
        let bs = cf[1].clone();
        check_msg(
            msg.channel_id
                .say(&ctx.http, format!("Card Price (USD): ${cp}"))
                .await,
        );
        check_msg(
            msg.channel_id
                .say(&ctx.http, fs.image_uris.unwrap().png.unwrap())
                .await,
        );
        check_msg(
            msg.channel_id
                .say(&ctx.http, bs.image_uris.unwrap().png.unwrap())
                .await,
        );
    } else {
        check_msg(
            msg.channel_id
                .say(&ctx.http, format!("Unable to find Card Price (USD)"))
                .await,
        );
        check_msg(msg.channel_id.say(&ctx.http, format!("Unable to find Card Image (possibly a single faced card). Here is Scryfall link:")).await);
        check_msg(msg.channel_id.say(&ctx.http, card.scryfall_uri).await);
    }

    let timeframe = now.elapsed().as_millis();

    check_msg(msg.channel_id.say(&ctx.http, format!("Fetch time in ms: {timeframe}\nModern: {card_legal_m:?}\nPauper: {card_legal_p:?}\nCommander: {card_legal_c:?}")).await);

    Ok(())
}


#[command]
#[aliases(lbw, wbl)]
#[only_in(guilds)]
async fn blackwhitelands(ctx: &Context, msg: &Message) -> CommandResult {
    let all_bw_lands = vec![
        "Neglected Manor",
        "Forlorn Flats",
        "Shadowy Backstreet",
        "Scoured Barrens",
        "Sunlit Marsh",
        "Concealed Courtyard",
        "Caves of Koilos",
        "Restless Fortress",
        "Godless Shrine",
        "Marsh Flats",
        "Temple of Silence",
        "Orzhov Guildgate",
        "Shattered Sanctum",
        "Silverquill Campus",
        "Shineshadow Snarl",
        "Snowfield Sinkhole",
        "Isolated Chapel",
        "Forsaken Sanctuary",
        "Shambling Vent",
        "Brightclimb Pathway",
        "Goldmire Bridge",
        "Silent Clearing",
        "Fetid Heath",
        "Orzhov Basilica",
    ];

    let x = all_bw_lands.join(", ");

    check_msg(msg.channel_id.say(&ctx.http, format!("{x}")).await);
    msg_clean_up(ctx, msg).await;

    Ok(())
}


#[command]
#[aliases(sr)]
#[only_in(guilds)]
async fn randomsearch(ctx: &Context, msg: &Message) -> CommandResult {
   msg_clean_up(ctx, msg).await;

    let now = Instant::now();
    let card = match Card::random().await {
        Ok(card) => card,
        Err(e) => panic!(
            "{}",
            format!(
                "{:?}",
                check_msg(msg.channel_id.say(&ctx.http, e.to_string()).await)
            )
        ),
    };

    let card_legal_m = card.legalities.modern;
    let card_legal_p = card.legalities.pauper;
    let card_legal_c = card.legalities.commander;
    let card_set = card.set_name;

    
	 let _card_price = match card.prices.usd {
        Some(card_price) => format!("{:?}", check_msg(
               msg.channel_id
                    .say(&ctx.http, format!("Random ass card found! (USD): ${card_price}"))
                    .await,
            )),
        None => format!(
            "{:?}",
            check_msg(
                msg.channel_id
                    .say(&ctx.http, format!("Unable to find Card Price (USD)"))
                    .await
            )
        ),
    };

    let card_image = match card.image_uris {
    	Some(card_image) => card_image.png,
    	None => panic!("{}", format!(
                "{:?}, {:?}",
                check_msg(msg.channel_id.say(&ctx.http, format!("Unable to find Card Image (Possibly a double faced card). Here is Scryfall link:")).await),
       			 check_msg(msg.channel_id.say(&ctx.http, card.scryfall_uri).await)
            ))
    };

    check_msg(msg.channel_id.say(&ctx.http, card_image.unwrap()).await);
    let timeframe = now.elapsed().as_millis();
    check_msg(msg.channel_id.say(&ctx.http, format!("Fetch time in ms: {timeframe}\nModern: {card_legal_m:?}\nPauper: {card_legal_p:?}\nCommander: {card_legal_c:?}\nPrinting: {card_set}")).await);


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