import React, { useState, useMemo } from 'react';
import { into, TIMEFRAME_CAMELCASE, indicatorLabels, indicatorColors } from '../types';
import type {
  TimeFrame,
  Risk,
  Style,
  Stance,
  CustomStrategy,
  TradeParams,
  AddMarketInfo,
  IndexId,
  IndicatorKind,
  AddMarketProps,
} from '../types';

const riskOptions: Risk[] = ['Low', 'Normal', 'High'];
const styleOptions: Style[] = ['Scalp', 'Swing'];
const stanceOptions: Stance[] = ['Bull', 'Bear', 'Neutral'];
const indicatorKinds: IndicatorKind[] = ['rsi', 'smaOnRsi', 'stochRsi', 'adx', 'atr', 'ema', 'emaCross', 'sma'];

export const AddMarket: React.FC<AddMarketProps> = ({ onClose, totalMargin }) => {
  const [asset, setAsset] = useState('');
  const [marginType, setMarginType] = useState<'alloc' | 'amount'>('alloc');
  const [marginValue, setMarginValue] = useState(0.1);
  const [tfSymbol, setTfSymbol] = useState<keyof typeof TIMEFRAME_CAMELCASE>('1m');
  const [lev, setLev] = useState(1);
  const [tradeTime, setTradeTime] = useState(0);
  const [risk, setRisk] = useState<Risk>('Low');
  const [style, setStyle] = useState<Style>('Scalp');
  const [stance, setStance] = useState<Stance>('Bull');
  const [followTrend, setFollowTrend] = useState(false);

  const [showConfig, setShowConfig] = useState(false);
  const [config, setConfig] = useState<IndexId[]>([]);

  const [newKind, setNewKind] = useState<IndicatorKind>('rsi');
  const [newParam, setNewParam] = useState(14);
  const [newParam2, setNewParam2] = useState(14);
  const [newTf, setNewTf] = useState<keyof typeof TIMEFRAME_CAMELCASE>('1m');

  const computedAmount = useMemo(
    () => (marginType === 'alloc' ? totalMargin * (marginValue / 100) : 0),
    [marginType, marginValue, totalMargin]
  );

 const handleAddIndicator = () => {
    let cfg: any;
    console.log(newKind);
    switch (newKind) {
      case 'emaCross':
        cfg = { emaCross: { short: newParam, long: newParam2 } };
        break;
      case 'smaOnRsi':
        cfg = { smaOnRsi: { periods: newParam, smoothing_length: newParam2 } };
        break;
      case 'stochRsi':
        cfg = { stochRsi: { periods: newParam, k_smoothing: null, dSmoothing: null } };
        break;
      case 'adx':
        cfg = { adx: { periods: newParam, di_length: newParam2 } };
        break;
      default:
        cfg = { [newKind]: newParam };
    }
    setConfig([...config, [cfg, newTf]]);
    setShowConfig(false);
  };

  const handleRemove = (i: number) => setConfig(config.filter((_, idx) => idx !== i));

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
                                   const validConfig = config.map(([ind, tf]) => [ind, into(tf)]);
    const info: AddMarketInfo = {
      asset: asset.toUpperCase(),
      marginAlloc: marginType === 'alloc' ? { alloc: marginValue / 100 } : { amount: marginValue },
      tradeParams: {
        timeFrame: into(tfSymbol as string) as TimeFrame,
        lev,
        strategy: { custom: { risk, style, stance, followTrend } } as CustomStrategy,
        tradeTime,
      } as TradeParams,
      config: validConfig,
    };

    console.log(JSON.stringify(validConfig));

    const res = await fetch('http://127.0.0.1:8090/command', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ addMarket: info }),
    });
    if (res.ok) onClose();
    else console.error('Submit failed');
  };

  const inputClass = 'mt-1 w-full border border-white bg-gray-600 text-white rounded px-3 py-2';
  const selectClass = 'mt-1 w-full border border-white bg-gray-600 text-white rounded px-3 py-2 cursor-pointer';
  const btnClass = 'px-5 py-2 border border-white bg-gray-600 text-white rounded hover:bg-gray-500 cursor-pointer';

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center backdrop-blur-sm">
      <form onSubmit={handleSubmit} className="relative bg-gray-600 rounded-2xl shadow-2xl w-full max-w-lg p-8 space-y-6 scale-90">
        <h2 className="text-2xl font-bold text-white">Add New Market</h2>
        <div className="text-sm text-white">Available Margin: <span className="font-semibold">{totalMargin.toFixed(2)}</span></div>
        <div className="grid grid-cols-2 gap-4">
          <div className="col-span-2">
            <label className="block text-sm text-white">Asset Symbol</label>
            <input type="text" value={asset} onChange={e => setAsset(e.target.value)} placeholder="e.g. BTC" required className={inputClass} />
          </div>
          <div>
            <label className="block text-sm text-white">Margin Type</label>
            <select value={marginType} onChange={e => setMarginType(e.target.value as any)} className={selectClass}>
              <option value="alloc">Percent</option>
              <option value="amount">Fixed</option>
            </select>
          </div>
          <div className="col-span-2">
            <label className="block text-sm text-white">{marginType === 'alloc' ? 'Percent' : 'Value'}</label>
            {marginType === 'alloc' ? (
              <>
                <input type="range" min={0} max={100} step={0.1} value={marginValue} onChange={e => setMarginValue(+e.target.value)} className="w-full h-2 bg-gray-200 cursor-pointer" />
                <div className="flex justify-between text-sm text-white mt-1"><span>0%</span><span>{marginValue.toFixed(1)}%</span><span>100%</span></div>
                <div className="text-sm text-white">Eq: {computedAmount.toFixed(2)}</div>
              </>
            ) : (
              <input type="number" step="any" value={marginValue} onChange={e => setMarginValue(+e.target.value)} className={inputClass} />
            )}
          </div>
          <div>
            <label className="block text-sm text-white">Time Frame</label>
            <select value={tfSymbol} onChange={e => setTfSymbol(e.target.value as any)} className={selectClass}>
              {Object.keys(TIMEFRAME_CAMELCASE).map(t => <option key={t} value={t}>{t}</option>)}
            </select>
          </div>
          <div>
            <label className="block text-sm text-white">Leverage</label>
            <input type="number" value={lev} onChange={e => setLev(+e.target.value)} min={1} className={inputClass} />
          </div>
          <div>
            <label className="block text-sm text-white">Trade Time (sec)</label>
            <input type="number" value={tradeTime} onChange={e => setTradeTime(+e.target.value)} min={0} className={inputClass} />
          </div>
        </div>
        <fieldset className="border-t border-white pt-4">
          <legend className="text-lg text-white">Strategy</legend>
          <div className="grid grid-cols-3 gap-4 mt-2">
            <div>
              <label className="block text-sm text-white">Risk</label>
              <select value={risk} onChange={e => setRisk(e.target.value as any)} className={selectClass}>
                {riskOptions.map(r => <option key={r} value={r}>{r}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-sm text-white">Style</label>
              <select value={style} onChange={e => setStyle(e.target.value as any)} className={selectClass}>
                {styleOptions.map(s => <option key={s} value={s}>{s}</option>)}
              </select>
            </div>
            <div>
              <label className="block text-sm text-white">Stance</label>
              <select value={stance} onChange={e => setStance(e.target.value as any)} className={selectClass}>
                {stanceOptions.map(s => <option key={s} value={s}>{s}</option>)}
              </select>
            </div>
            <div className="col-span-3 flex items-center mt-2">
              <input type="checkbox" checked={followTrend} onChange={e => setFollowTrend(e.target.checked)} className="h-4 w-4 text-white cursor-pointer" />
              <label className="ml-2 text-sm text-white cursor-pointer">Follow Trend</label>
            </div>
          </div>
        </fieldset>
        <fieldset className="mt-6 border-t border-white pt-6 relative">
          <legend className="text-lg text-white">Indicators</legend>
          <div className="flex flex-col gap-2">
            {config.map(([ind, tf], i) => {
              const kind = Object.keys(ind)[0] as IndicatorKind;
              return (
                <div key={i} className="flex items-center gap-2">
                  <span className={`${indicatorColors[kind]} px-3 py-1 rounded-full text-xs`}>{indicatorLabels[kind] || kind} -- {tf}</span>
                  <button type="button" onClick={() => handleRemove(i)} className="text-red-600 cursor-pointer">×</button>
                </div>
              );
            })}
            <button type="button" onClick={() => setShowConfig(true)} className="mt-2 text-sm text-white font-bold hover:underline cursor-pointer">Add Indicator</button>
          </div>
          {showConfig && (
            <div className="absolute bottom-10 left-full ml-4 w-64 bg-gray-600 border border-white rounded shadow p-4 z-20">
              <h3 className="text-sm font-semibold text-white">New Indicator</h3>
              <select value={newKind} onChange={e => setNewKind(e.target.value as IndicatorKind)} className={selectClass}>
                {indicatorKinds.map(k => <option key={k} value={k}>{indicatorLabels[k]}</option>)}
              </select>
              <div className="mt-2 grid grid-cols-2 gap-2">{(['emaCross','smaOnRsi','adx'].includes(newKind) ?
                <> <input type="number" value={newParam} onChange={e => setNewParam(+e.target.value)} placeholder="Param1" className={inputClass} />
                      <input type="number" value={newParam2} onChange={e => setNewParam2(+e.target.value)} placeholder="Param2" className={inputClass} /> </> :
                <input type="number" value={newParam} onChange={e => setNewParam(+e.target.value)} className={inputClass} />)}</div>
              <select value={newTf} onChange={e => setNewTf(e.target.value as any)} className={selectClass}>
                {Object.keys(TIMEFRAME_CAMELCASE).map(t => <option key={t} value={t}>{t}</option>)}
              </select>
              <div className="mt-4 flex justify-end gap-2">
                <button type="button" onClick={() => setShowConfig(false)} className="px-2 py-1 bg-gray-400 text-white rounded text-sm cursor-pointer">Cancel</button>
                <button type="button" onClick={handleAddIndicator} className="px-2 py-1 bg-gray-600 text-white rounded text-sm cursor-pointer">Add</button>
              </div>
            </div>
          )}
        </fieldset>
        <div className="flex justify-end gap-4 mt-6">
          <button type="button" onClick={onClose} className={btnClass}>Cancel</button>
          <button type="submit" className={btnClass}>Add Market</button>
        </div>
      </form>
    </div>
  );
};

