extern crate serenity;

mod commands;

use std::env;

use commands::*;

use serenity::{
    framework::{
        StandardFramework,
        standard::macros::group,
    },
    prelude::{
        EventHandler,
        Context,
    },
    model::{
        gateway,
        prelude::{Message, Activity, OnlineStatus},
    },
    Client,
    async_trait,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use serenity::model::prelude::GuildId;

const PREFIX: &'static str = ".";

#[group]
#[commands(ping, pfp, user, warn, play, skip, queue, osu)]
struct Command;

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
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix(PREFIX))
        .group(&COMMAND_GROUP);

    let mut client = Client::new(&token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<structs::MusicQueue>(Arc::new(RwLock::new(HashMap::<GuildId, Vec<String>>::new())));
        data.insert::<structs::VoiceManager>(Arc::clone(&client.voice_manager));
    }

    client.start().await.unwrap();
}
