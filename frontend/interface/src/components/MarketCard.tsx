import React from 'react';
import { motion } from 'framer-motion';
import { Pause, Play, Trash2 } from 'lucide-react';
import type { MarketInfo } from '../types';
import { indicatorLabels, indicatorColors, decompose, get_value, fromTimeFrame } from '../types';

interface MarketCardProps {
  market: MarketInfo;
  onTogglePause: (asset: string) => void;
  onRemove: (asset: string) => void;
}

const formatPrice = (n: number) => (n < 1 ? n.toFixed(4) : n.toFixed(2));

const PnlBar: React.FC<{ pnl: number }> = ({ pnl }) => {
  const w = Math.min(100, Math.abs(pnl));
  const pos = pnl >= 0;
  return (
    <div className="rounded-md border border-white/10 bg-black p-1">
      <div className="h-1.5 w-full bg-white/5">
        <div className={`${pos ? 'bg-orange-400' : 'bg-rose-500'}`} style={{ width: `${w}%`, height: '100%' }} />
      </div>
      <div className={`mt-1 text-right font-mono text-[11px] tabular-nums ${pos ? 'text-orange-300' : 'text-rose-300'}`}>{pos ? '+' : ''}{pnl.toFixed(2)}</div>
    </div>
  );
};

const MarketCard: React.FC<MarketCardProps> = ({ market, onTogglePause, onRemove }) => {
  const { asset, price, lev, margin, params, pnl, is_paused, indicators } = market;
  const { strategy } = params;
  const { risk, style, stance } = strategy.custom;

  return (
    <motion.div whileHover={{ y: -2 }} className="group rounded-md border border-white/10 bg-[#111316] p-4 shadow-[0_2px_0_rgba(255,255,255,0.03),_0_12px_24px_rgba(0,0,0,0.35)]">
      {/* Head */}
      <div className="mb-3 flex items-start justify-between">
        <div>
          <div className="text-[10px] uppercase text-white/50">Asset</div>
          <div className="-mt-0.5 flex items-baseline gap-3">
            <h2 className="text-3xl font-semibold tracking-tight">{asset}</h2>
            <span className={`relative bottom-1 rounded-md px-2 py-0.5 text-[10px] uppercase ${is_paused ? 'border border-amber-400/60 text-amber-300' : 'border border-orange-500/60 text-orange-300'}`}>{is_paused ? 'Paused' : 'Live'}</span>
          </div>
          <div className="mt-1 font-mono text-sm text-white/70">${formatPrice(price)} • {lev}×</div>
        </div>
        <div className="flex gap-2">
          <button onClick={() => onTogglePause(asset)} className="grid h-9 w-9 place-items-center rounded-md border border-white/10 bg-white/[0.04] hover:bg-white/10" title="Toggle">
            {is_paused ? <Play className="h-4 w-4 text-orange-300" /> : <Pause className="h-4 w-4 text-amber-300" />}
          </button>
          <button onClick={() => onRemove(asset)} className="grid h-9 w-9 place-items-center rounded-md border border-white/10 bg-white/[0.04] hover:bg-rose-600/20" title="Remove">
            <Trash2 className="h-4 w-4 text-rose-300" />
          </button>
        </div>
      </div>

      {/* Metrics */}
      <div className="grid grid-cols-3 gap-3">
        <div>
          <div className="text-[10px] uppercase text-white/50">Price</div>
          <div className="font-mono text-xl tabular-nums">${formatPrice(price)}</div>
        </div>
        <div>
          <div className="text-[10px] uppercase text-white/50">Leverage</div>
          <div className="font-mono text-xl">{lev}×</div>
        </div>
        <div>
          <div className="text-[10px] uppercase text-white/50">Margin</div>
          <div className="font-mono text-xl">${margin.toFixed(2)}</div>
        </div>
      </div>

      {/* PnL */}
      <div className="mt-4">
        <div className="text-[10px] uppercase text-white/50">PnL</div>
        <PnlBar pnl={pnl} />
      </div>

      {/* Indicators */}
      <div className="mt-3 flex flex-wrap gap-2">
        {indicators.map((data, i) => {
          const { kind, timeframe, value } = decompose(data);
          const kindKey = Object.keys(kind)[0] as keyof typeof indicatorColors;
          return (
            <span key={i} title={get_value(value)} className={`rounded-md border border-white/10 bg-white/5 px-2.5 py-1 text-[11px] ${indicatorColors[kindKey]}`}>
              {indicatorLabels[kindKey] || (kindKey as string)} — {fromTimeFrame(timeframe)}
            </span>
          );
        })}
      </div>

      {/* Strategy */}
      <div className="mt-4 grid grid-cols-3 gap-3 border-t border-white/10 pt-3 text-xs">
        <div>
          <div className="text-[10px] uppercase text-white/50">Strategy</div>
          <div className="truncate text-white/90">{style} / {stance}</div>
        </div>
        <div>
          <div className="text-[10px] uppercase text-white/50">Risk</div>
          <div className="text-white/90">{risk}</div>
        </div>
        <div className="text-right">
          <div className="text-[10px] uppercase text-white/50">Trend Following</div>
          <div className="text-white/90">{strategy.custom.followTrend ? 'Yes' : 'No'}</div>
        </div>
      </div>
    </motion.div>
  );
};

export default MarketCard;
