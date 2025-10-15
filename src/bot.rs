use log::info;
use tokio::time::{sleep, interval, Duration};
use std::collections::HashMap;
use hyperliquid_rust_sdk::{Error, InfoClient, Message,Subscription, TradeInfo as HLTradeInfo};
use crate::{Market,
    MarketCommand,
    MarketUpdate,AssetPrice,
    MARKETS,
    TradeParams,TradeInfo,
    Wallet, IndexId, LiquidationFillInfo,
    UpdateFrontend, AddMarketInfo, MarketInfo,
};

use crate::helper::{get_asset, subscribe_candles};
use tokio::{
    sync::mpsc::{Sender, UnboundedSender, UnboundedReceiver, unbounded_channel},
};

use crate::margin::{MarginAllocation, MarginBook, AssetMargin};
use crate::helper::address;
use std::sync::Arc;
use tokio::sync::Mutex;

use std::hash::BuildHasherDefault;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};


pub struct Bot{
    info_client: InfoClient,
    wallet: Arc<Wallet>,
    markets: HashMap<String, Sender<MarketCommand>, BuildHasherDefault<FxHasher>>,
    candle_subs: HashMap<String, u32>,
    session: Arc<Mutex<HashMap<String, MarketInfo, BuildHasherDefault<FxHasher>>>>,
    fees: (f64, f64),
    bot_tx: UnboundedSender<BotEvent>,
    bot_rv: UnboundedReceiver<BotEvent>,
    update_rv: Option<UnboundedReceiver<MarketUpdate>>,
    update_tx: UnboundedSender<MarketUpdate>,
    app_tx: Option<UnboundedSender<UpdateFrontend>>,
}



impl Bot{
    pub async fn new(wallet: Wallet) -> Result<(Self, UnboundedSender<BotEvent>), Error>{

        let mut info_client = InfoClient::with_reconnect(None, Some(wallet.url)).await?;
        let fees = wallet.get_user_fees().await?;

        let (bot_tx, mut bot_rv) = unbounded_channel::<BotEvent>();
        let (update_tx, mut update_rv) = unbounded_channel::<MarketUpdate>();

        Ok((Self{
            info_client, 
            wallet: wallet.into(),
            markets: HashMap::default(),
            candle_subs: HashMap::new(),
            session: Arc::new(Mutex::new(HashMap::default())),
            fees,
            bot_tx: bot_tx.clone(),
            bot_rv,
            update_rv: Some(update_rv),
            update_tx,
            app_tx: None,
        }, bot_tx))
    }


    pub async fn add_market(&mut self, info: AddMarketInfo, margin_book: &Arc<Mutex<MarginBook>>) -> Result<(), Error>{
       
        let AddMarketInfo {
            asset,
            margin_alloc,
            trade_params,
            config,
                } = info;
        let asset = asset.trim().to_uppercase();
        let asset_str = asset.as_str();

        if self.markets.contains_key(&asset){
            return Ok(());
        }

        if !MARKETS.contains(&asset_str){

            return Err(Error::AssetNotFound);
        }

        let mut book = margin_book.lock().await;
        let margin = book.allocate(asset.clone(), margin_alloc).await?;
        
        let meta = get_asset(&self.info_client, asset_str).await?;
        let (sub_id, mut receiver) = subscribe_candles(&mut self.info_client,
                                                        asset_str,
                                                        trade_params.time_frame.as_str())
                                                        .await?;

        
        let (market, market_tx) = Market::new(
            self.wallet.wallet.clone(),
            self.wallet.url,
            self.update_tx.clone(),
            receiver,
            meta,     
            margin,
            self.fees,
            trade_params,
            config,
        ).await?;


        self.markets.insert(asset.clone(), market_tx);
        self.candle_subs.insert(asset.clone(), sub_id);
        let cancel_margin = margin_book.clone();
        let app_tx = self.app_tx.clone(); 

        tokio::spawn(async move {
            if let Err(e) = market.start().await {
                if let Some(tx) = app_tx{
                    tx.send(UpdateFrontend::UserError(format!("Market {} exited with error: {:?}", &asset, e)));
                }
                let mut book = cancel_margin.lock().await;
                book.remove(&asset);
            }
        });         

        

        Ok(())

}


    pub async fn remove_market(&mut self, asset: &String, margin_book: &Arc<Mutex<MarginBook>>) -> Result<(), Error>{
        let asset = asset.trim().to_uppercase();
      
        if let Some(sub_id) = self.candle_subs.remove(&asset){
            let _ = self.info_client.unsubscribe(sub_id).await?;
            info!("Removed {} market successfully", asset);
        }else{
            info!("Couldn't remove {} market, it doesn't exist", asset);
            return Ok(());
        }

        if let Some(tx) = self.markets.remove(&asset){
            let tx = tx.clone();
            let cmd = MarketCommand::Close;
            let close = tokio::spawn(async move {
                if let Err(e) = tx.send(cmd).await{
                    log::warn!("Failed to send Close command: {:?}", e); 
                    return false;
                }
                true
            }).await.unwrap();
            
            if close{
                let mut sess_guard = self.session.lock().await;
                let _ = sess_guard.remove(&asset);
                let mut book = margin_book.lock().await;
                book.remove(&asset);
            }
        }else{
            info!("Failed: Close {} market, it doesn't exist", asset);
        }


        Ok(())
    }

    pub async fn pause_or_resume_market(&self, asset: &String){
        let asset = asset.trim().to_uppercase();
        
        if let Some(tx) = self.markets.get(&asset){
            let tx = tx.clone();
            let cmd = MarketCommand::Toggle;
            tokio::spawn(async move{
                if let Err(e) =  tx.send(cmd).await{
                    log::warn!("Failed to send Toggle command: {:?}", e);
                }
            });

        }else{
            info!("Failed: Pause {} market, it doesn't exist", asset);
            return;
        }

        let mut sess_guard = self.session.lock().await;
        if let Some(info) = sess_guard.get_mut(&asset){
            info.is_paused = !info.is_paused;
        }
    }

    pub async fn pause_all(&self){
       
        info!("PAUSING ALL MARKETS");
        for (_asset, tx) in &self.markets{
            let _ = tx.send(MarketCommand::Pause).await;
        }
        
        let mut session = self.session.lock().await;
        for (_asset, info) in session.iter_mut(){
           info.is_paused = true; 
        }
        
    }
    pub async fn resume_all(&self){
        info!("RESUMING ALL MARKETS");
        for (_asset, tx) in &self.markets{
            let _ = tx.send(MarketCommand::Resume).await;
        }
    }
    pub async fn close_all(&mut self){
        info!("CLOSING ALL MARKETS");
        for (_asset, id) in self.candle_subs.drain(){
                self.info_client.unsubscribe(id).await;
            } 
        self.candle_subs.clear();
        for (_asset, tx) in self.markets.drain(){
            let _ = tx.send(MarketCommand::Close).await;
        }
        
        let mut session = self.session.lock().await;
        session.clear();
    }


    pub async fn send_cmd(&self, asset: &String, cmd: MarketCommand){
        let asset = asset.trim().to_uppercase();
        
        if let Some(tx) = self.markets.get(&asset){
            let tx = tx.clone();
            tokio::spawn(async move{
                if let Err(e) =  tx.send(cmd).await{
                    log::warn!("Failed to send Market command: {:?}", e);
                }
            });
        }
}

    pub fn get_markets(&self) -> Vec<&String>{
        let mut assets = Vec::new();
         for (asset, _tx) in &self.markets{
            assets.push(asset);
        }

        assets
    }

    pub async fn get_session(&self) -> Vec<MarketInfo>{

        let mut guard = self.session.lock().await;
        let session: Vec<MarketInfo> = guard.values().cloned().collect();
        
        session
        
    }
    
 
    pub async fn start(mut self, app_tx: UnboundedSender<UpdateFrontend>) -> Result<(), Error>{
        use BotEvent::*; 
        use MarketUpdate::*;
        use UpdateFrontend::*;

        self.app_tx = Some(app_tx.clone());


        //safe
        let mut update_rv = self.update_rv.take().unwrap();
             
        
        let user = self.wallet.clone();
        let mut margin_book= MarginBook::new(user);
        let margin_arc = Arc::new(Mutex::new(margin_book)); 
        let margin_sync = margin_arc.clone();
        let margin_user_edit = margin_arc.clone();
        let margin_market_edit = margin_arc.clone();
        
        let app_tx_margin = app_tx.clone();
        let err_tx = app_tx.clone();

        //keep marginbook in sync with DEX 
        tokio::spawn(async move{
            let mut ticker = interval(Duration::from_secs(2));
           loop{
                ticker.tick().await;
                let result = {
                let mut book = margin_sync.lock().await;
                book.sync().await
                };

            match result {
                Ok(_) => {
                    let total = {
                        let book = margin_sync.lock().await;
                        book.total_on_chain - book.used()
                    };
                    let _ = app_tx_margin.send(UpdateTotalMargin(total));
                }
                Err(e) => {
                    log::warn!("Failed to fetch User Margin");
                    let _ = app_tx_margin.send(UserError(e.to_string()));
                    continue;
                }
            }
                let _ = sleep(Duration::from_millis(500)).await;
        } 
    });

        
        //Market -> Bot 
        let session_adder = self.session.clone();
        tokio::spawn(async move{
                while let Some(market_update) = update_rv.recv().await{

                    match market_update{
                        InitMarket(info) => {
                            let mut session_guard = session_adder.lock().await;
                            session_guard.insert(info.asset.clone(), info.clone());
                            let _ = app_tx.send(ConfirmMarket(info));     
                        },
                        PriceUpdate(asset_price) => {let _ = app_tx.send(UpdatePrice(asset_price));},
                        TradeUpdate(trade_info) => {
                            let _ = app_tx.send(NewTradeInfo(trade_info));
                            
                    },
                        MarginUpdate(asset_margin) => {
                            let result = {
                                let mut book = margin_market_edit.lock().await; 
                                book.update_asset(asset_margin.clone()).await
                            };

                            match result {
                                Ok(_) => {
                                    let _ = app_tx.send(UpdateMarketMargin(asset_margin));
                                }
                                Err(e) => {
                                    let _ = app_tx.send(UserError(e.to_string()));
                                }
                                }
                            },
                        RelayToFrontend(cmd) => {let _ = app_tx.send(cmd);
                        },
                    }
                }
        });

        //listen and send Liquidation events
            let (liq_tx, mut liq_rv) = unbounded_channel();
            let _id = self.info_client
                .subscribe(Subscription::UserFills{user: address(&self.wallet.pubkey) }, liq_tx)
                .await?;
        
        loop{
            tokio::select!(
                biased;

                Some(Message::UserFills(update)) = liq_rv.recv() => {

                    if update.data.is_snapshot.is_some(){
                        continue;
                    }
                    let mut liq_map: HashMap<String, Vec<HLTradeInfo>> = HashMap::new(); 

                    for trade in update.data.fills.into_iter(){
                        if trade.liquidation.is_some(){
                        liq_map
                            .entry(trade.coin.clone())
                            .or_insert_with(Vec::new)
                            .push(trade);
                        }
                    }
                    println!("\nTRADES  |||||||||| {:?}\n\n", liq_map);
        
                    for (coin, fills) in liq_map.into_iter(){
                        let to_send = LiquidationFillInfo::from(fills);
                        let cmd = MarketCommand::ReceiveLiquidation(to_send);
                        self.send_cmd(&coin, cmd).await;
                    }
            },


                Some(event) = self.bot_rv.recv() => {
            
                    match event{
                        AddMarket(add_market_info) => {
                            if let Err(e) = self.add_market(add_market_info, &margin_user_edit).await{
                                let _ = err_tx.send(UserError(e.to_string()));
                        }
                    },
                        ToggleMarket(asset) => {self.pause_or_resume_market(&asset).await;},
                        RemoveMarket(asset) => {let _ = self.remove_market(&asset, &margin_user_edit).await;},
                        MarketComm(command) => {self.send_cmd(&command.asset, command.cmd).await;},
                        ManualUpdateMargin(asset_margin) => {
                            let result = {
                                let mut book = margin_user_edit.lock().await;
                                book.update_asset(asset_margin.clone()).await
                            };
                            if let Ok(new_margin) = result{
                                let cmd = MarketCommand::UpdateMargin(new_margin);
                                self.send_cmd(&asset_margin.0.to_string(), cmd).await;
                            }
                        },
                        ResumeAll =>{self.resume_all().await},
                        PauseAll => {self.pause_all().await;},
                        CloseAll => {
                            self.close_all().await;
                            let mut book = margin_user_edit.lock().await;
                            book.reset();
                        },
                        
                        GetSession =>{
                            let session = self.get_session().await;
                            let _ = err_tx.send(LoadSession(session));   
                        },
                    }
            },

                
        )}

    }   

}




#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BotEvent{
    AddMarket(AddMarketInfo),
    ToggleMarket(String),
    RemoveMarket(String),
    MarketComm(BotToMarket),
    ManualUpdateMargin(AssetMargin),
    ResumeAll,
    PauseAll,
    CloseAll,
    GetSession, 
}



#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BotToMarket{
    pub asset: String,
    pub cmd: MarketCommand,
}







