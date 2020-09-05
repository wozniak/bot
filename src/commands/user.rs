use serenity::prelude::Context;
use serenity::model::prelude::{Message, User};
use serenity::framework::standard::CommandResult;
use serenity::prelude::Mentionable;
use serenity::framework::standard::macros::command;


#[command]
#[aliases(whois,userinfo)]
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

        let user = &mention;

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
                    .footer(|f| f.text(user.id))
            })
        }).await.unwrap();
    }

    Ok(())
}