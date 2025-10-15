use crate::TradeCommand;
//use crate::signal::IndicatorKind;
use kwant::indicators::Value;
use serde::{Deserialize, Serialize};
use crate::signal::ExecParams;

#[derive(Clone, Debug, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Risk {
    Low,
    Normal,
    High,
}

#[derive(Clone, Debug, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Style{
    Scalp,
    Swing,
}

#[derive(Clone, Debug, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Stance{
    Bull,
    Bear,
    Neutral,
}


#[derive(Clone, Debug, Copy, PartialEq,Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Strategy{
    Custom(CustomStrategy),
}

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomStrategy {
   pub risk: Risk,
   pub style: Style,    
   pub stance: Stance,
   pub follow_trend: bool,
}

pub struct RsiRange{
    pub low: f64,
    pub high: f64,
}

pub struct AtrRange{
    pub low: f64,
    pub high: f64,
}

pub struct StochRange{
    pub low: f64,
    pub high: f64,
}


impl CustomStrategy{

    pub fn new(risk: Risk, style: Style, stance: Stance, follow_trend: bool) -> Self{
        Self { risk, style, stance, follow_trend }
    }

    
    pub fn get_rsi_threshold(&self) -> RsiRange{
        match self.risk{
            Risk::Low => RsiRange{low: 25.0, high: 78.0},
            Risk::Normal => RsiRange{low: 30.0, high: 70.0},
            Risk::High => RsiRange{low: 33.0, high: 67.0},
        }
    }

    pub fn get_stoch_threshold(&self) -> StochRange{
        match self.risk{
            Risk::Low => StochRange{low: 2.0, high: 95.0},
            Risk::Normal => StochRange{low: 15.0, high: 85.0},
            Risk::High => StochRange{low:20.0, high: 80.0},
        }
    }


    pub fn get_atr_threshold(&self) -> AtrRange{
        match self.risk{
            Risk::Low => AtrRange{low: 0.2, high: 1.0},
            Risk::Normal => AtrRange{low: 0.5, high: 3.0},
            Risk::High => AtrRange{low: 0.8, high: f64::INFINITY},
        }
    }

    

    pub fn update_risk(&mut self, risk: Risk){
        self.risk = risk;
    }

    pub fn update_style(&mut self, style: Style){
        self.style = style;
    }

    pub fn update_direction(&mut self, stance: Stance){
        self.stance = stance;
    }
    
    pub fn update_follow_trend(&mut self, follow_trend: bool){
        self.follow_trend = follow_trend;
    }


    pub fn generate_signal(&self, data: Vec<Value>, price: f64, params: ExecParams) -> Option<TradeCommand> {
    // Extract indicator values from the data
    let mut rsi_value = None;
    let mut srsi_value = None;
    let mut stoch_rsi = None;
    let mut ema_cross = None;
    let mut adx_value = None;
    let mut atr_value = None;
    
    for value in data {
        match value {
            Value::RsiValue(rsi) => rsi_value = Some(rsi),
            Value::SmaRsiValue(srsi) => srsi_value = Some(srsi),
            Value::StochRsiValue { k, d } => stoch_rsi = Some((k, d)),
            Value::EmaCrossValue { short, long, trend } => {
                ema_cross = Some((short, long, trend))
            }
            Value::AdxValue(adx) => adx_value = Some(adx),
            Value::AtrValue(atr) => atr_value = Some(atr),
            _ => {} // Handle other indicators as needed
        }
    }
    
    //self.standard_strategy(rsi_value, stoch_rsi, ema_cross, adx_value, atr_value, price)
    if let Some(rsi) = rsi_value{
        if let Some(srsi) = srsi_value{
                if let Some(stoch) = stoch_rsi{
                    let max_size = (params.margin * params.lev as f64) / price;
                    return self.rsi_based_scalp(rsi, srsi, stoch, max_size);
                }
            }
    }

        None

       }


fn rsi_based_scalp(
    &self,
    rsi: f64,
    srsi: f64,
    stoch_rsi: (f64, f64), // (K, D)
    max_size: f64,
) -> Option<TradeCommand> {
    let (k, d) = stoch_rsi;
    let duration = 420;

    let rsi_dev = match self.risk {
        Risk::Low => 15.0,
        Risk::Normal => 30.0,
        Risk::High => 37.0,
    };

    const SRSI_OB: f64 = 80.0; 
    const SRSI_OS: f64 = 20.0; 

    if self.stance != Stance::Bull {
        let rsi_short = rsi > 100.0 - rsi_dev;
        let srsi_short = srsi > 100.0 - rsi_dev - 5.0;
        let stoch_short = k > SRSI_OB && d > SRSI_OB;

        if rsi_short && srsi_short && stoch_short {
            return Some(TradeCommand::ExecuteTrade {
                size: 0.9 * max_size,
                is_long: false,
                duration,
            });
        }
    }

    if self.stance != Stance::Bear {
        let rsi_long = rsi < rsi_dev;
        let srsi_long = srsi < rsi_dev + 5.0;
        let stoch_long = k < SRSI_OS && d < SRSI_OS;

        if rsi_long && srsi_long && stoch_long {
            return Some(TradeCommand::ExecuteTrade {
                size: 0.9 * max_size,
                is_long: true,
                duration,
            });
        }
    }

    None
}


fn standard_strategy(
    &self,
    rsi: Option<f64>,
    stoch_rsi: Option<(f64, f64)>,
    ema_cross: Option<(f64, f64, bool)>,
    adx: Option<f64>,
    atr: Option<f64>,
    price: f64,
) -> Option<TradeCommand> {
    // Scalping parameters
    const RSI_OVERSOLD: f64 = 30.0;
    const RSI_OVERBOUGHT: f64 = 70.0;
    const STOCH_OVERSOLD: f64 = 20.0;
    const STOCH_OVERBOUGHT: f64 = 80.0;
    const ADX_TREND_THRESHOLD: f64 = 25.0;
    const BASE_POSITION_SIZE: f64 = 0.1; // 10% of available capital
    const SCALP_DURATION: u64 = 300; // 5 minutes for scalping
    
    // Determine trend direction from EMA cross
    let trend_direction = if let Some((short_ema, long_ema, trend)) = ema_cross {
        if trend && short_ema > long_ema {
            Some(true) // Bullish trend
        } else if !trend && short_ema < long_ema {
            Some(false) // Bearish trend
        } else {
            None // No clear trend
        }
    } else {
        None
    };
    
    // Check trend strength with ADX
    let strong_trend = adx.map_or(true, |adx| adx > ADX_TREND_THRESHOLD);
    
    // Calculate position size based on ATR (volatility-adjusted sizing)
    let position_size = if let Some(atr) = atr {
        // Reduce position size in high volatility
        let volatility_adjustment = (atr / price).min(0.05); // Cap at 5%
        BASE_POSITION_SIZE * (1.0 - volatility_adjustment)
    } else {
        BASE_POSITION_SIZE
    };
    
    // Generate signals based on multiple confirmations
    
    // LONG SIGNAL LOGIC
    if let Some(true) = trend_direction {
        if strong_trend {
            // RSI-based long entry
            if let Some(rsi) = rsi {
                if rsi < RSI_OVERSOLD {
                    return Some(TradeCommand::ExecuteTrade {
                        size: position_size,
                        is_long: true,
                        duration: SCALP_DURATION,
                    });
                }
            }
            
            // StochRSI-based long entry (more sensitive for scalping)
            if let Some((k, d)) = stoch_rsi {
                if k < STOCH_OVERSOLD && d < STOCH_OVERSOLD && k > d {
                    // StochRSI bullish crossover in oversold region
                    return Some(TradeCommand::ExecuteTrade {
                        size: position_size,
                        is_long: true,
                        duration: SCALP_DURATION,
                    });
                }
            }
        }
    }
    
    // SHORT SIGNAL LOGIC
    if let Some(false) = trend_direction {
        if strong_trend {
            // RSI-based short entry
            if let Some(rsi) = rsi {
                if rsi > RSI_OVERBOUGHT {
                    return Some(TradeCommand::ExecuteTrade {
                        size: position_size,
                        is_long: false,
                        duration: SCALP_DURATION,
                    });
                }
            }
            
            // StochRSI-based short entry
            if let Some((k, d)) = stoch_rsi {
                if k > STOCH_OVERBOUGHT && d > STOCH_OVERBOUGHT && k < d {
                    // StochRSI bearish crossover in overbought region
                    return Some(TradeCommand::ExecuteTrade {
                        size: position_size,
                        is_long: false,
                        duration: SCALP_DURATION,
                    });
                }
            }
        }
    }
    
    // MOMENTUM SCALPING (when no clear EMA trend)
    if trend_direction.is_none() {
        // Use StochRSI for quick momentum plays
        if let Some((k, d)) = stoch_rsi {
            // Quick long on bullish momentum
            if k < 50.0 && d < 50.0 && k > d && (k - d) > 5.0 {
                if let Some(rsi) = rsi {
                    if rsi < 60.0 { // Don't buy into overbought
                        return Some(TradeCommand::ExecuteTrade {
                            size: position_size * 0.7, // Smaller size for momentum plays
                            is_long: true,
                            duration: SCALP_DURATION / 2, // Shorter duration
                        });
                    }
                }
            }
            
            // Quick short on bearish momentum
            if k > 50.0 && d > 50.0 && k < d && (d - k) > 5.0 {
                if let Some(rsi) = rsi {
                    if rsi > 40.0 { // Don't sell into oversold
                        return Some(TradeCommand::ExecuteTrade {
                            size: position_size * 0.7,
                            is_long: false,
                            duration: SCALP_DURATION / 2,
                        });
                    }
                }
            }
        }
    }
    
    None // No signal generated
}
}


impl Default for CustomStrategy{
    fn default() -> Self {
        Self { 
            risk: Risk::Normal,
            style: Style::Scalp,
            stance: Stance::Neutral,
            follow_trend: true,
    }
}
}













