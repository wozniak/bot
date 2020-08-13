extern crate serenity;

use std::env;
use serenity::client::Client;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::model::guild::BanOptions;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

const PREFIX: &'static str = ".";

#[group]
#[commands(ping, pfp, user, help, warn)]
struct General;

struct Handler;

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        println!("#{} {} {}", msg.channel(&ctx.cache).unwrap().guild().unwrap().read().name, msg.author.tag(), msg.content);
    }

    fn ready(&self, ctx: Context, ready: gateway::Ready) {
        println!("{} is connected!", ready.user.name);

        let activity = Activity::playing("game");

        ctx.set_presence(Some(activity), OnlineStatus::Online);
    }
}

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix(PREFIX)) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP));

    println!("starting client");
    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.description("pong!")
        })
    })?;

    Ok(())
}

#[command]
fn pfp(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mentions: Vec<User>;
    if msg.mentions == vec![] {
        mentions = vec![msg.author.clone()]; // only show the pfp of the message author
    } else {
        mentions = msg.mentions.clone(); // display the pfp of all of the people that the author mentioned
    }

    for mention in mentions {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{}'s avatar", mention.tag()))
                    .image(mention.face())
            })
        })?;
    }

    Ok(())
}

#[command]
fn warn(ctx: &mut Context, msg: &Message) -> CommandResult {
    let offender = &msg.mentions[0];

    if !msg.guild(&ctx.cache).unwrap()
            .read()
            .member(&ctx.http, msg.author.id)?
            .permissions(&ctx.cache)?
            .kick_members() {

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.description("you don't have the perms required to do that")
            })
        });

        return Ok(())
    }

    msg.channel_id.send_message(&ctx.http, |m| {


        m.embed(|e| {
            e.author(|f| {
                f.name(format!("{} was warned", offender.tag()))
                    .icon_url(offender.face())
            })
                .field("reason", &msg.content.split(' ').skip(2).collect::<Vec<_>>().join(" "), false)
                .footer(|f| f.text("powered by rust™"))
        })
    });

    Ok(())
}

#[command]
fn user(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mentions: Vec<User>;
    if msg.mentions == vec![] {
        mentions = vec![msg.author.clone()]; // only show the pfp of the message author
    } else {
        mentions = msg.mentions.clone(); // display the pfp of all of the people that the author mentioned
    }

    for mention in mentions {
        let member = msg.guild(&ctx.cache).unwrap().read()
            .member(&ctx, mention.id).unwrap();

        let mut roles: String = String::new();
        for role in member.roles(&ctx.cache).unwrap() {
            roles = format!("{} {}", role.mention(), roles);
        }

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.author(|f| {
                    f.name(mention.tag())
                        .icon_url(mention.face())
                })
                    .field("account creation", mention.created_at(), false)
                    .field("server join date", member.joined_at.unwrap(), false)
                    .field("roles", roles, false)
                    .thumbnail(mention.face())
                    .footer(|f| f.text("powered by rust™"))
            })
        });
    }

    Ok(())
}

#[command]
fn help(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.author(|f| {
                f.name(ctx.cache.read().user.tag());
                f.icon_url(ctx.cache.read().user.face())
            })
                .title("this bot's prefix is `.`")
                .field("user [user]", "shows information about a user", false)
                .field("pfp [user]", "shows the user's profile picture", false)
                .field("warn [user]", "warns a user (requires ban permission)", false)
        })
    });

    Ok(())
}