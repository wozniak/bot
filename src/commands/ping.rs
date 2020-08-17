use serenity::prelude::Context;
use serenity::model::prelude::{Message, User};
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.description("pong!")
        })
    }).await.unwrap();

    Ok(())
}