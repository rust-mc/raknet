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

    task::spawn(async move {
        let mut server = Server::new("0.0.0.0:19132".parse().unwrap(), None, None).await.unwrap();
        let mut incoming = server.incoming();
        while let Some((addr, packet)) = incoming.next().await {
            debug!("{}: {:?}", addr.to_string(), packet);
        }
    });

    loop {}
}
