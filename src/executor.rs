use std::sync::Arc;


use ethers::signers::LocalWallet;
use flume::Receiver;
use log::info;
use tokio::{
    sync::{mpsc::Sender, Mutex},
    time::{sleep, Duration},
};

use hyperliquid_rust_sdk::{
    Error,BaseUrl, ExchangeClient, ExchangeDataStatus, ExchangeResponseStatus, MarketOrderParams,
};

use crate::trade_setup::{TradeCommand, TradeFillInfo, TradeInfo, LiquidationFillInfo};
use crate::market::MarketCommand;




pub struct Executor {
    trade_rv: Receiver<TradeCommand>,
    market_tx: Sender<MarketCommand>,
    asset: String,
    exchange_client: Arc<ExchangeClient>,
    is_paused: bool,
    fees: (f64, f64),
    open_position: Arc<Mutex<Option<TradeFillInfo>>>,
}



impl Executor {

    pub async fn new(
        wallet: LocalWallet,
        asset: String,
        fees: (f64, f64),
        trade_rv: Receiver<TradeCommand>, 
        market_tx: Sender<MarketCommand>,
    ) -> Result<Executor, Error>{
        
        let exchange_client = Arc::new(ExchangeClient::new(None, wallet, Some(BaseUrl::Mainnet), None, None).await?);
        Ok(Executor{
            trade_rv,
            market_tx,
            asset,
            exchange_client,
            is_paused: false,
            fees,
            open_position: Arc::new(Mutex::new(None)),
        })
    }

    async fn try_trade(client: Arc<ExchangeClient>, params: MarketOrderParams<'_>) -> Result<ExchangeDataStatus, String>{

        let response = client
            .market_open(params)
            .await
            .map_err(|e| format!("Transport failure, {}",e))?;

        info!("Market order placed: {response:?}");

        let response = match response {
            ExchangeResponseStatus::Ok(exchange_response) => exchange_response,
            ExchangeResponseStatus::Err(e) => {
                return Err(format!("Exchange Error: Couldn't execute trade => {}",e));
         }
        };
        
        let status = response
            .data
            .filter(|d| !d.statuses.is_empty())
            .and_then(|d| d.statuses.get(0).cloned())
            .ok_or_else(|| "Exchange Error: Couldn't fetch trade status".to_string())?;

        Ok(status)

    }
    pub async fn open_order(&self,size: f64, is_long: bool) -> Result<TradeFillInfo, String>{
        
        let market_open_params = MarketOrderParams {
            asset: self.asset.as_str(),
            is_buy: is_long,
            sz: size as f64,
            px: None,
            slippage: Some(0.01), // 1% slippage
            cloid: None,
            wallet: None,
        };
        
        
        let status = Self::try_trade(self.exchange_client.clone(), market_open_params).await?;

         match status{
            
            ExchangeDataStatus::Filled(ref order) =>  {
            
                println!("Open order filled: {order:?}");
                let sz: f64 = order.total_sz.parse::<f64>().unwrap();
                let price: f64 = order.avg_px.parse::<f64>().unwrap(); 
                let fill_info = TradeFillInfo{fill_type: "Open".to_string(),sz, price, oid: order.oid, is_long};
                
                Ok(fill_info)
            },

            _ => Err("Open order not filled".to_string()),
            }


    }
    pub async fn close_order(&self, size: f64, is_long: bool) -> Result<TradeFillInfo, String>   {

        let market_close_params = MarketOrderParams{
            asset: self.asset.as_str(),
            is_buy: !is_long,
            sz: size as f64,
            px: None,
            slippage: Some(0.01), // 1% slippage
            cloid: None,
            wallet: None,
        };
        

        
        let status = Self::try_trade(self.exchange_client.clone(),market_close_params).await?;
        match status{

            ExchangeDataStatus::Filled(ref order) =>  {

                println!("Close order filled: {order:?}");
                let sz: f64 = order.total_sz.parse::<f64>().unwrap();
                let price: f64 = order.avg_px.parse::<f64>().unwrap(); 
                let fill_info = TradeFillInfo{fill_type: "Close".to_string(),sz, price, oid: order.oid, is_long};
                return Ok(fill_info);
            },

            _ => Err("Close order not filled".to_string()),
    }
    }


    pub async fn close_order_static(client: Arc<ExchangeClient>,asset: String, size: f64, is_long: bool) -> Result<TradeFillInfo, String>{
        
 
        let market_close_params = MarketOrderParams {
            asset: asset.as_str(),
            is_buy: !is_long,
            sz: size as f64,
            px: None,
            slippage: Some(0.01), // 1% slippage
            cloid: None,
            wallet: None,
        };

        let status = Self::try_trade(client,market_close_params).await?;
        match status{

            ExchangeDataStatus::Filled(ref order) =>  {

                println!("Close order filled: {order:?}");
                let sz: f64 = order.total_sz.parse::<f64>().unwrap();
                let price: f64 = order.avg_px.parse::<f64>().unwrap(); 
                let fill_info = TradeFillInfo{fill_type: "Close".to_string(),sz, price, oid: order.oid, is_long};
                return Ok(fill_info);
            },

            _ => Err("Close order not filled".to_string()),
    }


}


        fn get_trade_info(open: TradeFillInfo, close: TradeFillInfo, fees: &(f64, f64)) -> TradeInfo{
            let is_long = open.is_long;
            let (fee, pnl) = Self::calculate_pnl(fees,is_long, &open, &close);

            TradeInfo{
                open: open.price,
                close: close.price,
                pnl,
                fee,
                is_long, 
                duration: None,
                oid: (open.oid, close.oid),
            }
        }


     


    fn calculate_pnl(fees: &(f64, f64) ,is_long: bool, trade_fill_open: &TradeFillInfo, trade_fill_close: &TradeFillInfo) -> (f64, f64){
        let fee_open = trade_fill_open.sz * trade_fill_open.price * fees.1;
        let fee_close = trade_fill_close.sz * trade_fill_close.price * fees.1;
        
        let pnl = if is_long{
            trade_fill_close.sz * (trade_fill_close.price - trade_fill_open.price) - fee_open - fee_close
        }else{
            trade_fill_close.sz * (trade_fill_open.price - trade_fill_close.price) - fee_open - fee_close
        };

        (fee_open + fee_close, pnl)
    }


    pub async fn cancel_trade(&mut self) -> Option<TradeInfo>{

            if let Some(pos) = self.open_position.lock().await.take(){
                let trade_fill = self.close_order(pos.sz, pos.is_long).await;
                if let Ok(close) = trade_fill{
                    let trade_info = Self::get_trade_info(pos, close, &self.fees);
                    return Some(trade_info);
                }
        }
        
        None
    }

    async fn is_active(&self) -> bool{
        let guard = self.open_position.lock().await;
        guard.is_some()
    }

    fn toggle_pause(&mut self){
        self.is_paused = !self.is_paused
    }
    
    
    pub async fn start(mut self){
        println!("EXECUTOR STARTED");
             
            let info_sender = self.market_tx.clone();
            while let Ok(cmd) = self.trade_rv.recv_async().await{

                match cmd{
                        TradeCommand::ExecuteTrade {size, is_long, duration} => {
                                
                                if self.is_active().await || self.is_paused{continue};
                                let trade_info = self.open_order(size, is_long).await;
                                if let Ok(trade_fill) = trade_info{ 
                                    { 
                                        let mut pos = self.open_position.lock().await; 
                                        *pos = Some(trade_fill.clone()); 
                                    }         

                                    let client = self.exchange_client.clone();
                                    let asset = self.asset.clone();
                                    let fees = self.fees;
                                    let sender = info_sender.clone();
                                    let pos_handle = self.open_position.clone();
                                    tokio::spawn(async move{ 
                                        let _ = sleep(Duration::from_secs(duration)).await;
                                        let maybe_open = {
                                            let mut pos = pos_handle.lock().await;
                                            pos.take()
                                        }; 

                                        if let Some(open) = maybe_open{
                                          
                                            let close_fill = Self::close_order_static(client, asset, open.sz, is_long).await;
                                            if let Ok(fill) = close_fill{
                                                let trade_info = Self::get_trade_info(
                                                                                open,
                                                                                fill,
                                                                                &fees);
                                                  
                                                
                                                let _ = sender.send(MarketCommand::ReceiveTrade(trade_info)).await;
                                                info!("Trade Closed: {:?}", trade_info);
                                            }
                                    
                                    }
                        });
                                    };

                            },

                    TradeCommand::OpenTrade{size, is_long}=> {
                        info!("Open trade command received");

                            if !self.is_active().await && !self.is_paused{
                                 let trade_fill = self.open_order(size, is_long).await;

                                 if let Ok(trade) = trade_fill{
                                     info!("Trade Opened: {:?}", trade.clone());
                                     *self.open_position.lock().await = Some(trade);
                                    };
                    }else if self.is_active().await{
                        info!("OpenTrade skipped: a trade is already active");
                    }  


                },

                    TradeCommand::CloseTrade{size} => {
                            if self.is_paused{continue}; 
                            let maybe_open = {
                                let mut pos = self.open_position.lock().await;
                                pos.take()
                            };
                            
                            if let Some(open_pos) = maybe_open{
                                let size = size.min(open_pos.sz);
                                let trade_fill = self.close_order(size,open_pos.is_long).await;

                                if let Ok(fill) = trade_fill{
                                    let trade_info = Self::get_trade_info(
                                                        open_pos,
                                                        fill,
                                                        &self.fees);
                                    let _ = info_sender.send(MarketCommand::ReceiveTrade(trade_info)).await;
                                    info!("Trade Closed: {:?}", trade_info);
                            };
                        };
                },
 
                    TradeCommand::CancelTrade => {

                            if let Some(trade_info) = self.cancel_trade().await{
                                    let _ = info_sender.send(MarketCommand::ReceiveTrade(trade_info)).await;
                            };

                        return;

                    },
                    
                    TradeCommand::Liquidation(liq_fill) => {
                            let maybe_open = {
                                let mut pos = self.open_position.lock().await;
                                pos.take()
                            }; 
                            
                            if let Some(open_pos) = maybe_open{
                                let liq_fill: TradeFillInfo = liq_fill.into();
                                println!("MAKE SURE SIZES ARE THE SAME: \nLocal {open_pos:?}\nLiquidation: {liq_fill:?}");
                                let trade_info = Self::get_trade_info(
                                                    open_pos,
                                                    liq_fill,
                                                    &self.fees);
                                
                                    let _ = info_sender.send(MarketCommand::ReceiveTrade(trade_info)).await;
                                    info!("LIQUIDATION INFO: {:?}", trade_info);
                            }
                },

                    TradeCommand::Toggle=> {
                        
                        if let Some(trade_info) = self.cancel_trade().await{
                            let _ = info_sender.send(MarketCommand::ReceiveTrade(trade_info)).await;
                        };                        
                        self.toggle_pause();
                        info!("Executor is now {}", if self.is_paused { "paused" } else { "resumed" });
                },

                    TradeCommand::Pause => {
                        if let Some(trade_info) = self.cancel_trade().await{
                            let _ = info_sender.send(MarketCommand::ReceiveTrade(trade_info)).await;
                        }; 
                        self.is_paused = true;
                },
                    TradeCommand::Resume => {
                        self.is_paused = false;
                },
                    
                TradeCommand::BuildPosition{size, is_long, interval} => {info!("Contacting Bob the builder")},
                    
        }

    }}


}

