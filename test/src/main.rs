use async_std::stream::StreamExt;
use async_std::task;
use chrono::Local;
use env_logger::Builder;
use log::{debug, LevelFilter};

use raknet::*;

#[async_std::main]
async fn main() {
    Builder::new().format(|buf, record| {
        use std::io::Write;
        writeln!(
            buf,
            "[{}] [{}] - {}",
            Local::now().format("%Y-%m-%d | %H:%M:%S"),
            record.level(),
            record.args()
        )
    }).filter_level(LevelFilter::Debug).init();

    let t = task::spawn(async move {
        let mut listener = RakListener::bind("0.0.0.0:19132").await.unwrap();
        listener.start();
        let mut incoming = listener.incoming();
        while let Some((addr, packet)) = incoming.next().await {
            debug!("{}: {:?}", addr.to_string(), packet);
        }
    });

    loop {}
}
