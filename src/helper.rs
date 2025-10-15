use log::info;
use hyperliquid_rust_sdk::{Error,AssetMeta, InfoClient, Message, Subscription};
use tokio::sync::mpsc::{UnboundedReceiver};
use std::time::{SystemTime, UNIX_EPOCH};
use kwant::indicators::{Price};
use ethers::types::H160;
use crate::TimeFrame;
use log::warn;

pub async fn subscribe_candles(
    info_client: &mut InfoClient,
    coin: &str,
    tf: &str,
) -> Result<(u32,UnboundedReceiver<Message>), Error> {
    
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();


    let subscription_id = info_client
        .subscribe(
            Subscription::Candle {
                coin: coin.to_string(),
                interval: tf.to_string(),
            },
            sender,
        )
        .await?;
    info!("Subscribed to new candle data: {:?}", subscription_id);

    Ok((subscription_id, receiver))
}




fn get_time_now_and_candles_ago(candle_count: u64, tf: TimeFrame) -> (u64, u64) {
    let end = get_time_now();

    let interval = candle_count
    .checked_mul(tf.to_secs())
    .and_then(|s| s.checked_mul(1_000))
    .expect("interval overflowed");

    let start = end.saturating_sub(interval);

    (start, end)
}



async fn candles_snapshot(info_client: &InfoClient,coin: &str,time_frame: TimeFrame, start: u64, end: u64) -> Result<Vec<Price>, Error>{
 
    let vec = info_client
    .candles_snapshot(coin.to_string(), time_frame.to_string(), start, end)
    .await?;

    let mut res: Vec<Price> = Vec::with_capacity(vec.len());
    for candle in vec {
        let h = candle.high.parse::<f64>()
                .map_err(|e| Error::GenericParse(format!("Failed to parse high: {}", e)))?;
        let l = candle.low.parse::<f64>()
                .map_err(|e| Error::GenericParse(format!("Failed to parse low: {}", e)))?;
        let o = candle.open.parse::<f64>()
                .map_err(|e| Error::GenericParse(format!("Failed to parse open: {}", e)))?;
        let c = candle.close.parse::<f64>()
                .map_err(|e| Error::GenericParse(format!("Failed to parse close: {}", e)))?;

        res.push(Price {
            high: h,
            low: l,
            open: o,
            close: c,
    });
    }
    Ok(res)
}


pub async fn load_candles(info_client: &InfoClient,coin: &str,tf: TimeFrame, candle_count: u64) -> Result<Vec<Price>, Error> {


    let (start, end) = get_time_now_and_candles_ago(candle_count + 1, tf);

    let price_data = candles_snapshot(info_client, coin, tf, start, end).await?;

    Ok(price_data)
}




pub fn address(address: &String) -> H160 {
    address.parse().unwrap()

}



pub async fn get_max_lev(info_client: &InfoClient, token: &str) -> u32{
    let assets = info_client.meta().await.unwrap().universe;

    if let Some(asset) = assets.iter().find(|a| a.name == token) {
        asset.max_leverage
    }else{
        warn!("ERROR: Failed to retrieve max_leverage for {}", token);
        1
    }
}


pub async fn get_asset(info_client: &InfoClient, token: &str) -> Result<AssetMeta, Error>{
    let assets = info_client.meta().await?.universe;

    if let Some(asset) = assets.into_iter().find(|a| a.name == token) {
        Ok(asset)
    }else{
        return Err(Error::AssetNotFound);
    }
}

#[inline]
pub fn get_time_now() -> u64{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
    

