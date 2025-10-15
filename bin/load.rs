use hyperliquid_rust_sdk::{BaseUrl, InfoClient, AssetMeta};
use std::fs::File;
use std::io::Write;



#[tokio::main]
async fn main() {

    let mut info = InfoClient::new(None, Some(BaseUrl::Mainnet)).await.unwrap();

    let universe = info.meta().await.unwrap().universe;
    let assets: Vec<&str> = universe.iter().map(|a| a.name.as_str()).collect();

    let mut f = File::create("src/assets.rs").unwrap();

    f.write_all(format!("pub static MARKETS: &[&str] = &{:?};", assets).as_bytes());

}
