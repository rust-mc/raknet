use std::error::Error;
use async_std::task;
use raknet::*;

pub async fn async_main() -> Result<(), Box<dyn Error>> {
	let mut server = server::RakServer::bind("0.0.0.0:19132".parse()?).await?;
	server.listen().await;

	Ok(())
}

fn main() {
	task::block_on(async_main()).unwrap();
}