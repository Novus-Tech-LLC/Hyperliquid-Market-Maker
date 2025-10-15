use serde::{Deserialize, Serialize};
use crate::{MarketTradeInfo,MarginAllocation, IndexId, TradeParams, Value, AssetPrice, AssetMargin};
use std::collections::HashMap;


#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMarketInfo {
    pub asset: String,
    pub margin_alloc: MarginAllocation,
    pub trade_params: TradeParams,
    pub config: Option<Vec<IndexId>>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketInfo{
    pub asset: String,
    pub lev: u32,
    pub price: f64,
    pub params: TradeParams,
    pub margin: f64,
    pub pnl: f64,
    pub is_paused: bool,
    pub indicators: Vec<IndicatorData>,
}


#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorData{
    pub id: IndexId,
    pub value: Option<Value>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EditMarketInfo{
    Lev(f64),
    Strategy,
    Indicator(Vec<IndexId>),
}



#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateFrontend{
    ConfirmMarket(MarketInfo),
    UpdatePrice(AssetPrice),
    NewTradeInfo(MarketTradeInfo),
    UpdateTotalMargin(f64),
    UpdateMarketMargin(AssetMargin),
    UpdateIndicatorValues{asset: String, data: Vec<IndicatorData>},
    MarketInfoEdit((String, EditMarketInfo)),
    UserError(String),
    LoadSession(Vec<MarketInfo>),
}




