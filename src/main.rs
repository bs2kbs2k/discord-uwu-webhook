#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::env;

use serde_json::json;
use serenity::http::CacheHttp;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::iter::FromIterator;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.webhook_id.is_none() {
            let mut webhooks = match ctx
                .http
                .get_channel_webhooks(u64::from(msg.channel_id))
                .await
            {
                Err(why) => return eprintln!("Error getting webhooks: {:?}", why),
                Ok(list) => list,
            };
            let webhook = match if webhooks.is_empty() {
                match ctx
                    .http
                    .create_webhook(u64::from(msg.channel_id), &json!({"name": "UwU wats dis"}))
                    .await
                {
                    Err(why) => {
                        eprintln!("Error creating webhook: {:?}", why);
                        None
                    }
                    Ok(webhook) => Some(webhook),
                }
            } else {
                webhooks.pop()
            } {
                Some(webhook) => webhook,
                None => return,
            };
            let nick = msg
                .author_nick(&ctx)
                .await
                .unwrap_or(msg.author.name.clone());
            webhook
                .execute(&ctx, true, |mut w| {
                    w.avatar_url(
                        msg.author
                            .avatar_url()
                            .unwrap_or_else(|| msg.author.default_avatar_url()),
                    )
                    .content(uwuifier::uwuify_str_sse(&*msg.content))
                    .username(nick)
                })
                .await;
            msg.delete(&ctx).await;
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
