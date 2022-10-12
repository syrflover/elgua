use elgua::{cfg::Cfg, event, event::EventSender, handler::Handler, store::Store};
use log::LevelFilter;
use serenity::{framework::StandardFramework, prelude::*, Client};
use simple_logger::SimpleLogger;
use songbird::SerenityInit;
use tokio::sync::mpsc;

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
    let store = Store::connect(&cfg).await;
    let (event_tx, event_rx) = mpsc::channel(12);

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
        x.insert::<EventSender>(EventSender::new(event_tx))
    }

    tokio::select! {
        r = client.start() => {
            log::error!("{r:?}");
        }

        _ = event::process(event_rx) => {
            log::error!("error occured: event::process()");
        }
    };
}
