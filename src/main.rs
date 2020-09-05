extern crate serenity;
extern crate tokio;
extern crate toml;

mod commands;

use toml::Value;
use commands::*;
use serenity::{
    prelude::{
        EventHandler,
        Context,
    },
    framework::standard::{
        Args, CommandResult, CommandGroup,
        HelpOptions, help_commands,
        macros::{group, help},
    },
    model::{
        gateway,
        prelude::{GuildId, Message, Activity, OnlineStatus},
    },
    Client,
    async_trait,
};

use tokio::sync::Mutex;

use std::{
    sync::Arc,
    collections::HashMap,
};

use serenity::model::prelude::UserId;
use std::collections::HashSet;
use serenity::framework::StandardFramework;
use std::io::Read;

// built in help
#[help]
#[individual_command_tip = "hello! to get more info pass the command as an argument to help"]
#[command_not_found_text = "could not find the command `{}`"]
#[embed_success_colour = "BLURPLE"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Hide"]
#[wrong_channel = "Strike"]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    let _ = help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[group]
#[commands(ping, pfp, user, osu)]
struct General;


#[group]
#[commands(pay, bal, gamble)]
struct Economy;


#[group]
#[commands(purge, warn)]
struct Moderation;

#[group]
#[commands(play, skip, queue)]
struct Music;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        println!("#{} {} {}", msg.channel(&ctx.cache).await.unwrap().guild().unwrap().name, msg.author.tag(), msg.content);
    }

    async fn ready(&self, ctx: Context, ready: gateway::Ready) {
        println!("{} is connected!", ready.user.tag());

        let activity = Activity::playing("game");
        ctx.set_presence(Some(activity), OnlineStatus::Online).await;
    }
}


#[tokio::main]
async fn main() {
    let mut file = String::new();

    std::fs::File::open("Config.toml").unwrap().read_to_string(&mut file).unwrap();
    let config = file.parse::<Value>().unwrap();

    // sets up command group and prefix
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(config["config"]["prefix"].as_str().unwrap()))
        .help(&HELP)
        .group(&GENERAL_GROUP)
        .group(&MODERATION_GROUP)
        .group(&ECONOMY_GROUP)
        .group(&MUSIC_GROUP);

    let mut client = Client::new(config["tokens"]["discord"].as_str().unwrap())
        .event_handler(Handler)
        .framework(framework).await
        .expect("error creating client");


    // music-related guild variables
    {
        let mut data = client.data.write().await;
        data.insert::<structs::MusicQueue>  (Arc::new(Mutex::new(HashMap::<GuildId, Vec<String>>::new())));
        data.insert::<structs::MusicSkip>   (Arc::new(Mutex::new(HashMap::<GuildId, bool>::new())));
        data.insert::<structs::VoiceManager>(Arc::clone(&client.voice_manager));
        data.insert::<structs::Config>      (Arc::new(config));
        data.insert::<structs::Bank>        (Arc::new(Mutex::new(HashMap::<UserId, usize>::new())));

    }

    client.start()
        .await
        .expect("client crashed");
}
