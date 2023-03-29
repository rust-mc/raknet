use std::error::Error;
use async_std::task;
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use raknet::*;

pub async fn async_main() -> Result<(), Box<dyn Error>> {
	let mut server = server::RakServer::bind("0.0.0.0:19132".parse()?).await?;
	server.listen().await;

	Ok(())
}

fn main() {
	Builder::new()
		.format(|buf, record| {
			use std::io::Write;
			writeln!(buf,
			         "[{}] [{}] - {}",
			         Local::now().format("%Y-%m-%d | %H:%M:%S"),
			         record.level(),
			         record.args()
			)
		})
		.filter_level(LevelFilter::Debug)
		.init();

	task::block_on(async_main()).unwrap();
}