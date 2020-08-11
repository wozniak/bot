extern crate serenity;
extern crate scrap;
extern crate image;

use scrap::{Capturer, Display};
use std::io::ErrorKind::WouldBlock;
use std::fs::File;
use std::time::Duration;

use std::{env, thread};
use serenity::client::Client;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};
use enigo::Enigo;
use image::ImageEncoder;
use std::io::Write;

const PREFIX: &'static str = ".";

#[group]
#[commands(ping, pfp, user, screenshot)]
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
    let mut mentions: Vec<User>;
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
fn user(ctx: &mut Context, msg: &Message) -> CommandResult {
    let cache = serenity::cache::Cache::new();

    let mut mentions: Vec<User>;
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
                    f.name(mention.tag());
                    f.icon_url(mention.face())
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
fn screenshot(ctx: &mut Context, msg: &Message) -> CommandResult {
  let display = Display::primary().expect("Couldn't find primary display.");
    let mut capturer = Capturer::new(display).expect("Couldn't begin capture.");
    let (w, h) = (capturer.width(), capturer.height());

    msg.channel_id.send_message(&ctx.http, |m| { m.content("getting a frame") });
    loop {
        // Wait until there's a frame.

        let buffer = match capturer.frame() {
            Ok(buffer) => buffer,
            Err(error) => {
                if error.kind() == WouldBlock {
                    // Keep spinning.
                    continue;
                } else {
                    panic!("Error: {}", error);
                }
            }
        };

        msg.channel_id.send_message(&ctx.http, |m| { m.content("bitflipping") });

        let mut bitflipped = Vec::with_capacity(w * h * 4);
        let stride = buffer.len() / h;

        for y in 0..h {
            for x in 0..w {
                let i = stride * y + 4 * x;
                bitflipped.extend_from_slice(&[
                    buffer[i + 2],
                    buffer[i + 1],
                    buffer[i],
                    255,
                ]);
            }
        }

        msg.channel_id.send_message(&ctx.http, |m| { m.content("sending") });

        let mut file = File::create("tmp.png").unwrap();
        let png = image::png::PNGEncoder::new(&file);
        png.write_image(&bitflipped, w as u32, h as u32, image::ColorType::Rgba8).unwrap();
        file.flush();

        println!("took a screenshot");
        break
    }

    msg.channel_id.send_files(&ctx.http, vec!["tmp.png"], |m| { m.content("here lol") });
    Ok(())
}
