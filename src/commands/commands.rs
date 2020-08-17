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
use serenity::voice::Handler;

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
        {
            let data = ctx.data.read().await;
            let map = data.get::<MusicQueue>().unwrap().read().await;
            let queue = map.get(&msg.guild_id.unwrap()).unwrap();
            if queue.last().unwrap() == &url {
                break;
            }
        }
        sleep(std::time::Duration::new(2, 0));
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
    {
        let data = ctx.data.read().await;

        let manager_lock = data.get::<VoiceManager>().cloned().unwrap();
        let mut manager = manager_lock.lock().await;

        let guild = msg.guild(&ctx.cache).await.unwrap();
        let channel = guild.voice_states.get(&msg.author.id).unwrap();

        let handler = manager.join(msg.guild_id.unwrap(), channel.channel_id.unwrap()).unwrap();
        handler.stop();
    }

    println!("waiting for write handle");

    match ctx.data.write().await.get::<MusicQueue>().unwrap().write().await.get_mut(&msg.guild_id.unwrap()) {
        Some(v) => { v.pop(); },
        None => {
            msg.channel_id.send_message(&ctx.http, |f| {
                f.embed(|e| {
                    e.description("nothing in queue")
                })
            }).await.unwrap();
        }
    }

    msg.channel_id.send_message(&ctx.http, |f| {
        f.embed(|e| {
            e.description("music stopped")
        })
    }).await.unwrap();

    Ok(())

}