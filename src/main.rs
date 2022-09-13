use elgua::{cfg::Cfg, handler::Handler, store::Store};
use serenity::{framework::StandardFramework, prelude::*, Client};
use songbird::SerenityInit;

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    let cfg = Cfg::new();
    let store = Store::connect(&cfg.database_url).await;

    let framework = StandardFramework::new();

    let intents = GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&cfg.token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .await
        .unwrap();

    {
        let mut x = client.data.write().await;
        x.insert::<Cfg>(cfg);
        x.insert::<Store>(store);
    }

    client.start().await.unwrap();
}
