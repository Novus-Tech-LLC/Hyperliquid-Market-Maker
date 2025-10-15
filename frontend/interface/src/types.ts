export type IndicatorKind =
  | { rsi: number }
  | { smaOnRsi: { periods: number; smoothingLength: number } }
  | { stochRsi: { periods: number; kSmoothing?: number | null; dSmoothing?: number | null } }
  | { adx: { periods: number; diLength: number } }
  | { atr: number }
  | { ema: number }
  | { emaCross: { short: number; long: number } }
  | { sma: number };

export interface MarketInfo{
    asset: string, 
    lev: number,
    price: number,
    margin: number,
    params: TradeParams,
    pnl: number,
    is_paused: boolean,
    indicators: indicatorData[],
    trades: TradeInfo[],
}

export interface indicatorData {
    id: IndexId,
    value?: Value
};

export type Value =
  | { rsiValue: number }
  | { stochRsiValue: { k: number; d: number } }
  | { emaValue: number }
  | { emaCrossValue: { short: number; long: number; trend: boolean } }
  | { smaValue: number }
  | { smaRsiValue: number }
  | { adxValue: number }
  | { atrValue: number }

export function get_value(v?: Value): string {
  if (!v) return "No value";
  if ("rsiValue" in v) return `RSI: ${v.rsiValue.toFixed(2)}`;
  if ("stochRsiValue" in v) return `StochRSI: K=${v.stochRsiValue.k.toFixed(2)}, D=${v.stochRsiValue.d.toFixed(2)}`;
  if ("emaValue" in v) return `EMA: ${v.emaValue.toFixed(2)}`;
  if ("emaCrossValue" in v) return `EMA Cross: short=${v.emaCrossValue.short.toFixed(2)}, long=${v.emaCrossValue.long.toFixed(2)}, trend=${v.emaCrossValue.trend ? "↑" : "↓"}`;
  if ("smaValue" in v) return `SMA: ${v.smaValue.toFixed(2)}`;
  if ("smaRsiValue" in v) return `SMA on RSI: ${v.smaRsiValue.toFixed(2)}`;
  if ("adxValue" in v) return `ADX: ${v.adxValue.toFixed(2)}`;
  if ("atrValue" in v) return `ATR: ${v.atrValue.toFixed(2)}`;
  return "Unknown";
}

export type Decomposed = {
  kind: IndicatorKind
  timeframe: TimeFrame
  value?: Value
}



export function decompose(ind: indicatorData): Decomposed {
  const [kind, timeframe] = ind.id
  return { kind, timeframe, value: ind.value }
}

export type IndexId = [IndicatorKind, TimeFrame];


export type TimeFrame =
  | "min1"
  | "min3"
  | "min5"
  | "min15"
  | "min30"
  | "hour1"
  | "hour2"
  | "hour4"
  | "hour12"
  | "day1"
  | "day3"
  | "week"
  | "month";

export const TIMEFRAME_CAMELCASE: Record<string, TimeFrame> = {
  "1m": "min1",
  "3m": "min3",
  "5m": "min5",
  "15m": "min15",
  "30m": "min30",
  "1h": "hour1",
  "2h": "hour2",
  "4h": "hour4",
  "12h": "hour12",
  "1d": "day1",
  "3d": "day3",
  "w": "week",
  "m": "month",
};

const TIMEFRAME_SHORT: Record<TimeFrame, string> = Object.entries(TIMEFRAME_CAMELCASE)
  .reduce((acc, [short, tf]) => {
    acc[tf] = short;
    return acc;
  }, {} as Record<TimeFrame, string>);

export function fromTimeFrame(tf: TimeFrame): string {
  return TIMEFRAME_SHORT[tf];
}

export function into(tf: string): TimeFrame {
  return TIMEFRAME_CAMELCASE[tf];
}




export type Risk = "Low" | "Normal" | "High";
export type Style = "Scalp" | "Swing";
export type Stance = "Bull" | "Bear" | "Neutral";

export interface CustomStrategy {
  risk: Risk;
  style: Style;
  stance: Stance;
  followTrend: boolean;
}

export type Strategy = { custom: CustomStrategy };

export interface TradeParams {
  timeFrame: TimeFrame;  
  lev: number;
  strategy: Strategy;
  tradeTime: number;
}

export type MarginAllocation =
  | {alloc: number }
  | {amount: number };




export interface AddMarketInfo {
  asset: string;
  marginAlloc: MarginAllocation;
  tradeParams: TradeParams;
  config?: IndexId[];
};

export type Message = 
    | { confirmMarket: MarketInfo }
    | { updatePrice: assetPrice }
    | { newTradeInfo: MarketTradeInfo }
    | { updateTotalMargin: number}
    | { updateMarketMargin: assetMargin }
    | { updateIndicatorValues: {asset: string, data: indicatorData[] }}
    | { marketInfoEdit: [string, editMarketInfo]}
    | { userError: string }
    | { loadSession: MarketInfo[]};


export type assetPrice = [string, number];
export type assetMargin = [string, number];


export type editMarketInfo = 
    | {lev: number}
    | {strategy: Strategy}
    | {margin: number};


export interface TradeInfo{
    open: number,
    close: number,
    pnl: number,
    fee: number,
    is_long: number,
    duration?: number,
    oid: [number, number]
};


export interface MarketTradeInfo{
    asset: string,
    info: TradeInfo,
}


export const indicatorLabels: Record<string, string> = {
  rsi: 'RSI',
  smaOnRsi: 'SMA on RSI',
  stochRsi: 'Stoch RSI',
  adx: 'ADX',
  atr: 'ATR',
  ema: 'EMA',
  emaCross: 'EMA Cross',
  sma: 'SMA',
};

export const indicatorColors: Record<string, string> = {
  rsi: 'bg-green-800 text-green-200',
  smaOnRsi: 'bg-indigo-800 text-indigo-200',
  stochRsi: 'bg-purple-800 text-purple-200',
  adx: 'bg-yellow-800 text-yellow-200',
  atr: 'bg-red-800 text-red-200',
  ema: 'bg-blue-800 text-blue-200',
  emaCross: 'bg-pink-800 text-pink-200',
  sma: 'bg-gray-800 text-gray-200',
};









