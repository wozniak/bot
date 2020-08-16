use serenity::{
    prelude::*,
    voice,
    model::{
        prelude::*,
    },
    framework::standard::{
        CommandResult,
        macros::command,
    },
};

use crate::commands::structs::*;
use std::ops::Deref;
use std::thread::sleep;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.description("pong!")
        })
    }).await.unwrap();

    Ok(())
}

#[command]
async fn pfp(ctx: &Context, msg: &Message) -> CommandResult {
    let users: Vec<User>;

    if !(msg.mentions == vec![]) {
        users = msg.mentions.clone();
    } else {
        users = vec![msg.author.clone()];
    }

    for user in users {
        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{}'s avatar", user.tag()))
                    .image(user.face())
            })
        }).await?;
    }

    Ok(())
}

#[command]
async fn warn(ctx: &Context, msg: &Message) -> CommandResult {
    let offender = &msg.mentions[0];

    if !msg.member(&ctx.cache).await.unwrap()
            .permissions(&ctx.cache).await?
            .kick_members() {

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.description("you don't have the perms required to do that")
            })
        }).await.unwrap();

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
    }).await.unwrap();

    Ok(())
}

#[command]
async fn user(ctx: &Context, msg: &Message) -> CommandResult {
    let mentions: Vec<User>;
    if msg.mentions == vec![] {
        mentions = vec![msg.author.clone()]; // only show the pfp of the message author
    } else {
        mentions = msg.mentions.clone(); // display the pfp of all of the people that the author mentioned
    }

    for mention in mentions {
        let member = msg.guild(&ctx.cache).await.unwrap()
            .member(&ctx, mention.id).await.unwrap();

        let mut roles: String = String::new();
        for role in member.roles(&ctx.cache).await.unwrap() {
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
        }).await.unwrap();
    }

    Ok(())
}

#[command]
async fn play(ctx: &Context, msg: &Message) -> CommandResult {
    let url: String = msg.content.split(" ").skip(1).collect::<Vec<_>>().join(" ");

    // queues song
    {
        let mut data = ctx.data.write().await;
        let mut queue = data.get_mut::<MusicQueue>().unwrap().write().await;

        match queue.get_mut(&msg.guild_id.unwrap()) {
            Some(v) => v.insert(0, url.clone()),
            None => { queue.insert(msg.guild_id.unwrap(), vec![url.clone()]); },
        }
    }

    msg.channel_id.send_message(&ctx.http, |f| {
        f.embed(|e| {
            e.description(format!("queued '{}'", &url))
        })
    }).await.unwrap();

    loop {
        sleep(std::time::Duration::new(2, 0));
        let data = ctx.data.read().await;
        let map = data.get::<MusicQueue>().unwrap().read().await;
        let queue = map.get(&msg.guild_id.unwrap()).unwrap();
        if queue.last().unwrap() == &url {
            break;
        }
    }

    println!("it is time! starting!");

    let data = ctx.data.read().await;
    let mut manager = data.get::<VoiceManager>().unwrap().lock().await;
    let mut handler = manager.join(msg.guild_id.unwrap(),
                                   msg.guild(&ctx.cache)
                                       .await.unwrap()
                                       .voice_states
                                       .get(&msg.author.id).unwrap()
                                       .channel_id.unwrap()
    ).unwrap();

    let source = voice::ytdl_search(&url).await.unwrap();

    handler.stop();
    handler.play(source);

    println!("playing...");

    Ok(())
}

#[command]
async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    let l = ctx.data.read().await.get::<MusicQueue>().cloned().unwrap();
    let x = l.read().await;
    let songs = x.get(&msg.guild_id.unwrap()).unwrap().iter();

    msg.channel_id.send_message(&ctx.http, |f| {
        f.embed(move |mut e| {
            e = e.title("queue");
            for song in songs {
                e = e.field("song", song, false)
            }
            e
        })
    }).await.unwrap();

    Ok(())
}

#[command]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;

    let manager_lock = data.get::<VoiceManager>().cloned().unwrap();
    let mut manager = manager_lock.lock().await;

    let guild = msg.guild(&ctx.cache).await.unwrap();
    let channel = guild.voice_states.get(&msg.author.id).unwrap();

    let handler = manager.join(msg.guild_id.unwrap(), channel.channel_id.unwrap()).unwrap();

    ctx.data.write().await.get::<MusicQueue>().unwrap().write().await.get_mut(&msg.guild_id.unwrap()).unwrap().pop();
    handler.stop();

    msg.channel_id.send_message(&ctx.http, |f| {
        f.embed(|e| {
            e.description("music stopped")
        })
    }).await.unwrap();

    Ok(())

}

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {

    let current_user = ctx.cache.current_user().await;

    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.author(|f| {
                f.name(current_user.tag())
                    .icon_url(current_user.face())
            })
                .title("this bot's prefix is `.`")
                .field("user [user]", "shows information about a user", false)
                .field("pfp [user]", "shows the user's profile picture", false)
                .field("warn [user]", "warns a user (requires kick permission)", false)
        })
    }).await.unwrap();

    Ok(())
}