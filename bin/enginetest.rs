#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::{
    env, fs,
    str::FromStr,
};

//
use dotenv::dotenv;
use log::info;
use std::sync::Arc;
use hyperliquid_rust_sdk::Error;
use hyperliquid_rust_bot::{
    Bot,
    BotEvent,
    MarginAllocation,
    AssetMargin,
    BotToMarket,
    MarketCommand,
    IndexId, Entry, EditType, IndicatorKind,
    MARKETS,
    TradeParams,TimeFrame, AddMarketInfo, UpdateFrontend,

    LocalWallet, Wallet, BaseUrl,
};
use hyperliquid_rust_bot::strategy::{Strategy, CustomStrategy, Risk, Style, Stance};

use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{sleep, Duration},
};


const COIN: &str = "BTC";
const URL: BaseUrl = BaseUrl::Mainnet;

#[tokio::main]
async fn main() -> Result<(), Error>{
    use IndicatorKind::*;
    env_logger::init();
    match URL{
        BaseUrl::Mainnet => dotenv().ok(),
        BaseUrl::Testnet => dotenv::from_filename("testnet").ok(),
        BaseUrl::Localhost => dotenv::from_filename(".env.test").ok(),
        };
        let wallet = load_wallet(BaseUrl::Mainnet).await?;
        let strat = Strategy::Custom(load_strategy("./config.toml"));
        
        let trade_params = TradeParams{
        strategy: strat,
        lev: 20,
        trade_time: 300,
        time_frame: TimeFrame::from_str("5m").unwrap_or(TimeFrame::Min1),
    
    };

    let config = Vec::from([
    (
        IndicatorKind::Rsi(12),
        TimeFrame::Min1,
    ),
        (
        IndicatorKind::SmaOnRsi{periods: 14, smoothing_length: 9},
        TimeFrame::Hour1,
    ),
    (
        IndicatorKind::StochRsi{periods: 16,k_smoothing: Some(4), d_smoothing: Some(4)},
        TimeFrame::Hour4,
    ),
      
    (
        IndicatorKind::EmaCross{short: 21, long: 200},
        TimeFrame::Day1,
    ),  
    (
        IndicatorKind::Adx {
            periods: 14,
            di_length: 14,
        },
        TimeFrame::Min5,
    ),
    (
        IndicatorKind::Atr(14),
        TimeFrame::Min15,
    ),
    (
        IndicatorKind::Sma(50),
        TimeFrame::Hour1,
    ),
]);
    
    let (app_tx, mut app_rv) = unbounded_channel::<UpdateFrontend>();

    let (mut bot, sender) = Bot::new(wallet).await?;

    tokio::spawn(async move {
        bot.start(app_tx).await;
    });

    

    tokio::spawn(async move {
     let market_add = AddMarketInfo{
        asset: COIN.to_string(), 
        margin_alloc: MarginAllocation::Alloc(0.1),
        trade_params: trade_params.clone(),
        config: Some(config),
    };
        let market_add2 = AddMarketInfo{
        asset: "SOL".to_string(), 
        margin_alloc: MarginAllocation::Alloc(0.1),
        trade_params: TradeParams::default(),
        config: None,
    };
        let market_add3 = AddMarketInfo{
        asset: "xrp ".to_string(), 
        margin_alloc: MarginAllocation::Amount(50.0),
        trade_params: trade_params,
        config: None,
    };
        let cmd = BotToMarket{
            asset:"BTC".to_string(),
            cmd: MarketCommand::UpdateLeverage(40),
        };

        let cmd2 = BotToMarket{
            asset: "SOL".to_string(),
            cmd: MarketCommand::EditIndicators(Vec::from([Entry{id: (Ema(33), TimeFrame::Hour4),edit: EditType::Add}])),
        };

        let cmd3 = BotToMarket{
            asset: "XRP".to_string(),
            cmd: MarketCommand::EditIndicators(Vec::from([Entry{id: (Ema(33), TimeFrame::Hour4),edit: EditType::Toggle},
                                                            Entry{id: (Rsi(12), TimeFrame::Min1),edit: EditType::Add},
                                                        ])
                                                ),
        };
   
        let _ = sleep(Duration::from_secs(5)).await;
        sender.send(BotEvent::AddMarket(market_add.clone()));
        let _ = sleep(Duration::from_secs(5)).await;
        sender.send(BotEvent::AddMarket(market_add2));
        sender.send(BotEvent::AddMarket(market_add3));
        //let _ = sleep(Duration::from_secs(20)).await;
        //sender.send(BotEvent::RemoveMarket("BTC".to_string()));
        let _ = sleep(Duration::from_secs(5)).await;
        sender.send(BotEvent::MarketComm(cmd));
        sender.send(BotEvent::MarketComm(cmd2));
        let _ = sleep(Duration::from_secs(5)).await;
        sender.send(BotEvent::MarketComm(cmd3));
        let _ = sleep(Duration::from_secs(10)).await;

        sender.send(BotEvent::PauseAll);
        let _ = sleep(Duration::from_secs(10)).await;
        sender.send(BotEvent::ResumeAll);
        let _ = sleep(Duration::from_secs(10)).await;
        sender.send(BotEvent::CloseAll);
        let _ = sleep(Duration::from_secs(10)).await;
        sender.send(BotEvent::AddMarket(market_add));
    });


    while let Some(update) = app_rv.recv().await{
            info!("FRONT END RECEIVED {:?}", update);
    }

/*   tokio::spawn(async move{
        
        /*let _ = sleep(Duration::from_secs(10)).await;
        sender.send(MarketCommand::UpdateLeverage(50)).await;
        let _ = sleep(Duration::from_secs(10)).await;
        sender.send(MarketCommand::UpdateLeverage(40)).await;*/

        //let _ = sleep(Duration::from_secs(120)).await;
        //sender.send(MarketCommand::Pause).await;
        let _ = sleep(Duration::from_secs(20)).await;
        //sender.send(MarketCommand::UpdateTimeFrame(TimeFrame::from_str("4h").unwrap())).await;
        let _ = sender.send(MarketCommand::EditIndicators(Vec::from([Entry{id: (Ema(33), TimeFrame::Hour1),edit: EditType::Add}, 
                                                            Entry{id: (SmaOnRsi{periods: 12, smoothing_length: 9}, TimeFrame::Min1),edit: EditType::Add}
        ]))).await;

        let _ = sleep(Duration::from_secs(20)).await;
        let _ = sender.send(MarketCommand::EditIndicators(Vec::from([Entry{id: (Ema(33), TimeFrame::Hour4),edit: EditType::Add}]))).await;
        let _ = sleep(Duration::from_secs(10)).await;
        let _ = sender.send(MarketCommand::EditIndicators(Vec::from([Entry{id: (Ema(33), TimeFrame::Hour1),edit: EditType::Toggle}]))).await;

        let _ = sleep(Duration::from_secs(20)).await;
        let _ = sender.send(MarketCommand::EditIndicators(Vec::from([Entry{id: (Sma(10), TimeFrame::Min5),edit: EditType::Add}]))).await;
        let _ = sleep(Duration::from_secs(20)).await;
        sender.send(MarketCommand::EditIndicators(Vec::from([Entry{id: (Ema(10), TimeFrame::Hour4),edit: EditType::Remove}, 
                                                            Entry{id: (Sma(10), TimeFrame::Min5),edit: EditType::Remove},
                                                            Entry{id: (Atr(14), TimeFrame::Min15),edit: EditType::Remove},
                                                            Entry{id: (Rsi(12), TimeFrame::Min1),edit: EditType::Toggle}
        ]))).await;
        let _ = sleep(Duration::from_secs(30)).await;
        let _ = sender.send(MarketCommand::EditIndicators(Vec::from([]))).await;
        let _ =sender.send(MarketCommand::EditIndicators(Vec::from([Entry{id: (Rsi(12), TimeFrame::Min1),edit: EditType::Toggle}]))).await;
        let _ = sleep(Duration::from_secs(100000)).await;
        sender.send(MarketCommand::Close).await;
        //let _ = sleep(Duration::from_secs(30)).await;
        //let _ = sender.send(MarketCommand::Close).await; 
});
*/
    
    Ok(())
}




fn load_strategy(path: &str) -> CustomStrategy {
    let content = fs::read_to_string(path).expect("failed to read file");
    toml::from_str(&content).expect("failed to parse toml")
}


async fn load_wallet(url: BaseUrl) -> Result<Wallet, Error>{
    let wallet = std::env::var("PRIVATE_KEY").expect("Error fetching PRIVATE_KEY")
        .parse();

    if let Err(ref e) = wallet{
        return Err(Error::Custom(format!("Failed to load wallet: {}", e))); 
    }
    let pubkey: String = std::env::var("WALLET").expect("Error fetching WALLET address");
    Ok(Wallet::new(url , pubkey, wallet.unwrap()).await?)
}








