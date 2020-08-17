use serenity::prelude::Context;
use serenity::model::prelude::Message;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::command;


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