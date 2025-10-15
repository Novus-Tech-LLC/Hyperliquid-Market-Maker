use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;

use std::hash::BuildHasherDefault;
use rustc_hash::FxHasher;

use arraydeque::{ArrayDeque, behavior::Wrapping};
use kwant::indicators::{Rsi, Atr, StochasticRsi, Price, Indicator, Ema, EmaCross, Sma, SmaRsi, Adx, Value};

use crate::trade_setup::TimeFrame;
use crate::helper::get_time_now;
use crate::MAX_HISTORY;
use crate::IndicatorData;

use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone)]
pub struct ExecParams{
    pub margin: f64,
    pub lev: u32,
    pub tf: TimeFrame,
} 

impl ExecParams{
    pub fn new(margin: f64, lev:u32, tf: TimeFrame)-> Self{
       Self{
            margin,
            lev,
            tf,
        } 
    }
}

pub enum ExecParam{
    Margin(f64),
    Lev(u32),
    Tf(TimeFrame),
}


#[derive(Debug, Clone,Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IndicatorKind{
    Rsi(u32),
    SmaOnRsi{periods: u32, smoothing_length: u32},
    StochRsi{periods: u32, k_smoothing: Option<u32>, d_smoothing: Option<u32>},
    Adx{periods: u32, di_length: u32},
    Atr(u32),
    Ema(u32),
    EmaCross{short:u32, long:u32},
    Sma(u32),
}

#[derive(Debug)]
pub struct Handler{
    pub indicator: Box<dyn Indicator>,
    pub is_active: bool,
}

impl Handler{

    pub fn new(indicator: IndicatorKind) -> Handler{
        Handler{
            indicator: match_kind(indicator),
            is_active: true,
        }
    }

    fn toggle(&mut self) -> bool{
        self.is_active = !self.is_active;
        self.is_active
    }

    pub fn update(&mut self,price: Price, after_close: bool){
        if !after_close{
            self.indicator.update_before_close(price);
        }else{
            self.indicator.update_after_close(price);
        }
    }
    pub fn get_value(&self) -> Option<Value>{
        self.indicator.get_last()
    }

    pub fn load<'a,I: IntoIterator<Item=&'a Price>>(&mut self, price_data: I){
        let data_vec: Vec<Price> = price_data.into_iter().copied().collect();
        self.indicator.load(&data_vec);
    }

    pub fn reset(&mut self){
        self.indicator.reset();
    }
 
}


unsafe impl Send for Handler {}


pub type IndexId = (IndicatorKind, TimeFrame);

fn match_kind(kind: IndicatorKind) -> Box<dyn Indicator> {
    match kind {
        IndicatorKind::Rsi (periods) => {
            Box::new(Rsi::new(periods, periods, None,None, None))
        }
         IndicatorKind::SmaOnRsi{periods, smoothing_length} => {
            Box::new(SmaRsi::new(periods, smoothing_length))
        }
        IndicatorKind::StochRsi{periods, k_smoothing, d_smoothing}=> {
            Box::new(StochasticRsi::new(periods, k_smoothing, d_smoothing))
        }
        IndicatorKind::Adx { periods, di_length } => {
            Box::new(Adx::new(periods, di_length))
        }
        IndicatorKind::Atr(periods) => {
            Box::new(Atr::new(periods))
        }
        IndicatorKind::Ema(periods) => {
            Box::new(Ema::new(periods))
        }
        IndicatorKind::EmaCross { short, long } => {
            Box::new(EmaCross::new(short, long))
        }
        IndicatorKind::Sma(periods) => {
            Box::new(Sma::new(periods))
        }
    }
}


type History = Box<ArrayDeque<Price, MAX_HISTORY, Wrapping>>;

#[derive(Debug)]
pub struct Tracker{
    pub price_data: History,
    pub indicators: HashMap<IndicatorKind, Handler, BuildHasherDefault<FxHasher>>,
    tf: TimeFrame,
    next_close: u64,
}



impl Tracker{
    pub fn new(tf: TimeFrame) -> Self{
        Tracker{
            price_data: Box::new(ArrayDeque::new()),
            indicators: HashMap::default(),
            tf,
            next_close: Self::calc_next_close(tf),
        }
    }


    pub fn digest(&mut self, price: Price){
        let time = get_time_now(); 
       
        if time >= self.next_close{
            self.next_close = Self::calc_next_close(self.tf);
            self.price_data.push_back(price);
            self.update_indicators(price, true);
        }else{
            self.update_indicators(price, false);
        }
        
    }

    fn update_indicators(&mut self,price: Price, after_close: bool){

        for (_kind, handler) in &mut self.indicators{
            handler.update(price, after_close);
        }
    }
    
    fn calc_next_close(tf: TimeFrame)-> u64 {
        let now = get_time_now();

        let tf_ms = tf.to_millis();
        ((now / tf_ms) + 1) * tf_ms
    }
    
    
    pub async fn load<I: IntoIterator<Item=Price>>(&mut self, price_data: I){
        let buffer: Vec<Price> = price_data.into_iter().collect();
        let safe_buff: Arc<[Price]> = buffer.clone().into();

        let mut handles: Vec<tokio::task::JoinHandle<(IndicatorKind, Handler)>> = Vec::new();
        let mut temp_handlers = std::mem::take(&mut self.indicators);
    
        for (kind, mut handler) in temp_handlers{
            let buff = safe_buff.clone();

            let handle = tokio::spawn(async move{
                handler.load(&*buff);
                (kind, handler)
            });

            handles.push(handle);
        }

        let new_indicators: HashMap<IndicatorKind, Handler, BuildHasherDefault<FxHasher>> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(Result::unwrap) // unwrap JoinHandle
        .collect();     
        
        self.indicators = new_indicators;
        self.price_data.extend(buffer);
    }


    pub fn add_indicator(&mut self, kind: IndicatorKind, load: bool){
        let mut handler = Handler::new(kind);
        if load{
            handler.load(&*self.price_data);
        }
        self.indicators.insert(kind, handler);
    }

    pub fn remove_indicator(&mut self, kind: IndicatorKind){
        self.indicators.remove(&kind);
    } 

    pub fn toggle_indicator(&mut self, kind: IndicatorKind){
        if let Some(handler) = self.indicators.get_mut(&kind){
            let _ = handler.toggle();
        }
    }

    pub fn get_active_values(&self) -> Vec<Value>{
        let mut values = Vec::new();
        for (_kind, handler) in &self.indicators{
            if let Some(val) = handler.get_value(){
                values.push(val);
            }
        }
        values
    }

    
    pub fn get_indicators_data(&self) -> Vec<IndicatorData>{
        let mut values = Vec::new();
        for (kind, handler) in &self.indicators{
            if let Some(val) = handler.get_value(){
                values.push(
                    IndicatorData{
                        id: (*kind, self.tf),
                        value: Some(val),
                    }
                );
            }
        }
        values
    }
  
    pub fn reset(&mut self){
        self.price_data.clear();
        for (_kind, handler) in &mut self.indicators{
            handler.reset();
        }
    }
    
}



pub type TimeFrameData = HashMap<TimeFrame, Vec<Price>>;

#[derive(Copy, Clone, Debug,PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry{
    pub id: IndexId,
    pub edit: EditType
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EditType{
    Toggle,
    Add,
    Remove,
}







