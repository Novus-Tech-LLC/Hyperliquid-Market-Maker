use std::fmt;

use log::info;
use hyperliquid_rust_sdk::{ExchangeClient, ExchangeResponseStatus, Error, TradeInfo as HLTradeInfo};
//use kwant::indicators::Price;

use crate::strategy::{Strategy, CustomStrategy};
use serde::{Deserialize, Serialize};



#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeParams {
    pub strategy: Strategy, 
    pub lev: u32,
    pub trade_time: u64,  
    pub time_frame: TimeFrame,
}



impl TradeParams{

    pub async fn update_lev(&mut self, lev: u32, client: &ExchangeClient, asset: &str, first_time: bool) -> Result<u32, Error>{   
            if !first_time && self.lev == lev{
                return Err(Error::Custom(format!("Leverage is unchanged")));
            }
            
            let response = client
            .update_leverage(lev, asset, false, None)
            .await?;

            info!("Update leverage response: {response:?}");
            match response{
                ExchangeResponseStatus::Ok(_) => {
                    self.lev = lev;
                    return Ok(lev);
            },
                ExchangeResponseStatus::Err(e)=>{
                    return Err(Error::Custom(e));
            },
        }
    }

}



impl Default for TradeParams {
    fn default() -> Self {
        Self {
            strategy: Strategy::Custom(CustomStrategy::default()),
            lev: 20,
            trade_time: 300,
            time_frame: TimeFrame::Min5,
        }
    }
}

impl fmt::Display for TradeParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "leverage: {}\nStrategy: {:?}\nTrade time: {} s\ntime_frame: {}",
            self.lev,
            self.strategy,
            self.trade_time,
            self.time_frame.as_str(),
        )
    }
}


#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TradeCommand{
    ExecuteTrade {size: f64, is_long: bool, duration: u64},
    OpenTrade {size: f64, is_long: bool},
    CloseTrade{size: f64},
    BuildPosition {size: f64, is_long: bool, interval: u64},
    CancelTrade,
    Liquidation(LiquidationFillInfo),
    Toggle,
    Resume,
    Pause,
}


#[derive(Clone, Debug, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeInfo{
    pub open: f64,
    pub close: f64,
    pub pnl: f64,
    pub fee: f64,
    pub is_long: bool,
    pub duration: Option<u64>,
    pub oid: (u64, u64),
}



#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketTradeInfo{
    pub asset: String,
    pub info: TradeInfo,
}




#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeFillInfo{
    pub price: f64,
    pub fill_type: String,
    pub sz: f64,
    pub oid: u64,  
    pub is_long: bool, }

impl From<LiquidationFillInfo> for TradeFillInfo{

    fn from(liq: LiquidationFillInfo) -> Self{
        let LiquidationFillInfo {price, sz, oid, is_long} = liq;

        TradeFillInfo{
            price,
            fill_type: "Liquidation".to_string(),
            sz,
            oid,
            is_long,
        }
    } 
}


#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidationFillInfo{
    pub price: f64,
    pub sz: f64,
    pub oid: u64,  
    pub is_long: bool, //was the user going long ?
}



impl From<Vec<HLTradeInfo>> for LiquidationFillInfo{

    fn from(trades: Vec<HLTradeInfo>) -> Self{
        let n = trades.len();
        let is_long = match trades[0].side.as_str(){
            "A" => true,
            "B" => false,
            _ => panic!("THIS IS INSANE"),
        };

        let mut sz: f64 = f64::from_bits(1);
        let mut total: f64 = f64::from_bits(1);
        
        trades.iter().for_each(|t| {
            let size = t.sz.parse::<f64>().unwrap();
            total += size * t.px.parse::<f64>().unwrap(); 
            sz += size;
        });

        let avg_px = total / sz;
         
        Self{
            price: avg_px,
            sz,
            oid: 000000,
            is_long,
        }   
    }
}






//TIME FRAME
#[derive(Debug, Clone, Copy, PartialEq, Eq,Deserialize,Serialize, Hash)]
#[serde(rename_all = "camelCase")]
pub enum TimeFrame {
    Min1,
    Min3,
    Min5,
    Min15,
    Min30,
    Hour1,
    Hour2,
    Hour4,
    Hour12,
    Day1,
    Day3,
    Week,
    Month,
}




impl TimeFrame{
    
    pub fn to_secs(&self) -> u64{
        match *self {
            TimeFrame::Min1   => 1 * 60,
            TimeFrame::Min3   => 3 * 60,
            TimeFrame::Min5   => 5 * 60,
            TimeFrame::Min15  => 15 * 60,
            TimeFrame::Min30  => 30 * 60,
            TimeFrame::Hour1  => 1 * 60 * 60,
            TimeFrame::Hour2  => 2 * 60 * 60,
            TimeFrame::Hour4  => 4 * 60 * 60,
            TimeFrame::Hour12 => 12 * 60 * 60,
            TimeFrame::Day1   => 24 * 60 * 60,
            TimeFrame::Day3   => 3 * 24 * 60 * 60,
            TimeFrame::Week   => 7 * 24 * 60 * 60,
            TimeFrame::Month  => 30 * 24 * 60 * 60, // approximate month as 30 days
        }
    }

    pub fn to_millis(&self) -> u64{
        self.to_secs() * 1000
    }


}

impl TimeFrame {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeFrame::Min1   => "1m",
            TimeFrame::Min3   => "3m",
            TimeFrame::Min5   => "5m",
            TimeFrame::Min15  => "15m",
            TimeFrame::Min30  => "30m",
            TimeFrame::Hour1  => "1h",
            TimeFrame::Hour2  => "2h",
            TimeFrame::Hour4  => "4h",
            TimeFrame::Hour12 => "12h",
            TimeFrame::Day1   => "1d",
            TimeFrame::Day3   => "3d",
            TimeFrame::Week   => "w",
            TimeFrame::Month  => "m",
        }
    }
    pub fn to_string(&self) -> String{
        
        self.as_str().to_string()

    } 

}

impl std::fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}



impl std::str::FromStr for TimeFrame {

    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {

        match s {
            "1m"  => Ok(TimeFrame::Min1),
            "3m"  => Ok(TimeFrame::Min3),
            "5m"  => Ok(TimeFrame::Min5),
            "15m" => Ok(TimeFrame::Min15),
            "30m" => Ok(TimeFrame::Min30),
            "1h"  => Ok(TimeFrame::Hour1),
            "2h"  => Ok(TimeFrame::Hour2),
            "4h"  => Ok(TimeFrame::Hour4),
            "12h" => Ok(TimeFrame::Hour12),
            "1d"  => Ok(TimeFrame::Day1),
            "3d"  => Ok(TimeFrame::Day3),
            "w"   => Ok(TimeFrame::Week),
            "m"   => Ok(TimeFrame::Month),
         _     => Err(format!("Invalid TimeFrame string: '{}'", s)),
        }
    }
}



