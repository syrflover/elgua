use elgua::{cfg::Cfg, event, event::EventSender, handler::Handler, store::Store};
use log::LevelFilter;
use serenity::{prelude::*, Client};
use simple_logger::SimpleLogger;
use songbird::SerenityInit;
use tokio::{
    fs,
    signal::{self, unix::SignalKind},
    sync::mpsc,
};

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

    // TODO: 업데이트 구현
    // 이미 있는 것보다 최신 버전만 받아오도록
    youtube_dl::download_yt_dlp(".").await.unwrap();
    fs::create_dir_all(elgua::audio::YTDL_CACHE).await.unwrap();
    fs::create_dir_all(elgua::audio::SCDL_CACHE).await.unwrap();

    let cfg = Cfg::new();
    let store = Store::connect(&cfg).await;
    let (event_tx, event_rx) = mpsc::channel(12);

    let intents = GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&cfg.token, intents)
        .event_handler(Handler)
        .register_songbird()
        .await
        .unwrap();

    {
        let mut x = client.data.write().await;
        x.insert::<Cfg>(cfg);
        x.insert::<Store>(store);
        x.insert::<EventSender>(EventSender::new(event_tx))
    }

    let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        r = client.start() => {
            log::error!("{r:?}");
        }

        _ = event::process(event_rx) => {
            log::error!("error occured: event::process()");
        }

        _ = sigterm.recv() => {},

        _ = async { signal::ctrl_c().await.expect("failed to listen for ctrl_c event") } => {}
    };
}
