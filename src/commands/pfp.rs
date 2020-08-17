use serenity::prelude::Context;
use serenity::model::prelude::{Message, User};
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;

#[command]
pub async fn pfp(ctx: &Context, msg: &Message) -> CommandResult {
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