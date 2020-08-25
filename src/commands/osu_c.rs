use serenity::{
    prelude::*,
    model::{
        prelude::*,
    },
    framework::standard::{
        CommandResult,
        macros::command,
    },
};

use std::env;
use rosu::Osu;

#[command]
async fn osu(ctx: &Context, msg: &Message) -> CommandResult {
    let username = msg.content.split(" ").skip(1).collect::<Vec<_>>().join(" ");

    let osu = Osu::new(&env::var("OSU_TOKEN").unwrap());
    let user_request = rosu::backend::UserRequest::with_username(username).unwrap();
    let user = match user_request.queue_single(&osu).await.unwrap() {
        Some(u) => u,
        None => {
            msg.channel_id.send_message(&ctx.http, |m| m.content("no such user")).await.unwrap();
            return Ok(());
        }
    };

    msg.channel_id.send_message(&ctx.http, |m| m.embed(|e| {
        e.author(|a| {
            a.name(&user.username)
                .icon_url(format!("http://s.ppy.sh/a/{}", user.user_id))
                .url(format!("https://osu.ppy.sh/users/{}", user.user_id))
        })
            .field("PP",        user.pp_raw,    true)
            .field("country",   user.country,   true)
            .field("level",     user.level,     true)
            .field("SS count",  user.count_ss,  true)
            .field("S count",   user.count_s,   true)
    })).await.unwrap();

    Ok(())
}