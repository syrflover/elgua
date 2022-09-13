use elgua::{cfg::Cfg, handler::Handler, store::Store};
use log::LevelFilter;
use serenity::{framework::StandardFramework, prelude::*, Client};
use simple_logger::SimpleLogger;
use songbird::SerenityInit;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    SimpleLogger::new()
        .with_level(LevelFilter::Debug)
        // .with_local_timestamps()
        .with_module_level("tracing", LevelFilter::Error)
        .with_module_level("serenity::gateway::ws_client_ext", LevelFilter::Error)
        .init()
        .unwrap();

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
