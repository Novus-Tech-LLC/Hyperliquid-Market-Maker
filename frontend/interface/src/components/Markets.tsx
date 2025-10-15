import React, { useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Plus, Power, Pause, X, AlertCircle } from 'lucide-react';
import MarketCard from './MarketCard';
import { AddMarket } from './AddMarket';
import type { MarketInfo, Message, assetPrice, MarketTradeInfo, assetMargin, indicatorData } from '../types';

export default function MarketsPage() {
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const errRef = useRef<NodeJS.Timeout | null>(null);

  const [markets, setMarkets] = useState<MarketInfo[]>([]);
  const [totalMargin, setTotalMargin] = useState(0);
  const [marketToRemove, setMarketToRemove] = useState<string | null>(null);
  const [marketToToggle, setMarketToToggle] = useState<string | null>(null);
  const [showAdd, setShowAdd] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectRef = useRef<number>();

  useEffect(() => {
    if (wsRef.current) return;
    const connect = () => {
      const ws = new WebSocket('ws://localhost:8090/ws');
      wsRef.current = ws;
      ws.onopen = () => load_session();
      ws.onmessage = (event: MessageEvent) => {
        const payload = JSON.parse(event.data) as Message;
        if ('confirmMarket' in payload) {
          setMarkets(prev => [
            ...prev,
            { ...payload.confirmMarket, trades: Array.isArray(payload.confirmMarket.trades) ? payload.confirmMarket.trades : [] },
          ]);
        } else if ('updatePrice' in payload) {
          const [asset, price] = payload.updatePrice as assetPrice;
          setMarkets(prev => prev.map(m => (m.asset === asset ? { ...m, price } : m)));
        } else if ('newTradeInfo' in payload) {
          const { asset, info } = payload.newTradeInfo as MarketTradeInfo;
          setMarkets(prev => prev.map(m => (m.asset === asset ? { ...m, trades: [...(Array.isArray(m.trades) ? m.trades : []), info], pnl: (m.pnl += info.pnl) } : m)));
        } else if ('updateTotalMargin' in payload) {
          setTotalMargin(payload.updateTotalMargin);
        } else if ('updateMarketMargin' in payload) {
          const [asset, margin] = payload.updateMarketMargin as assetMargin;
          setMarkets(prev => prev.map(m => (m.asset === asset ? { ...m, margin } : m)));
        } else if ('updateIndicatorValues' in payload) {
          const { asset, data } = payload.updateIndicatorValues as { asset: string; data: indicatorData[] };
          setMarkets(prev => prev.map(m => (m.asset === asset ? { ...m, indicators: data } : m)));
        } else if ('userError' in payload) {
          setErrorMsg(payload.userError);
          if (errRef.current) clearTimeout(errRef.current);
          errRef.current = setTimeout(() => setErrorMsg(null), 5000);
        } else if ('loadSession' in payload) {
          setMarkets(payload.loadSession);
        }
      };
      ws.onerror = err => console.error('WebSocket error', err);
      ws.onclose = () => {
        reconnectRef.current = window.setTimeout(connect, 1000);
      };
    };
    connect();
    return () => {
      if (reconnectRef.current) clearTimeout(reconnectRef.current);
      wsRef.current?.close();
    };
  }, []);

  const remove_market = async (asset: string) => {
    await fetch('http://localhost:8090/command', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ removeMarket: asset.toUpperCase() }),
    });
  };
  const toggle_market = async (asset: string) => {
    await fetch('http://localhost:8090/command', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ toggleMarket: asset.toUpperCase() }),
    });
  };
  const load_session = async () => {
    await fetch('http://localhost:8090/command', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ getSession: null }),
    });
  };

  const handleConfirmToggle = (asset: string, isPaused: boolean) => {
    if (isPaused) {
      toggle_market(asset);
      setMarkets(prev => prev.map(m => (m.asset === asset ? { ...m, is_paused: false } : m)));
    } else setMarketToToggle(asset);
  };
  const handleTogglePause = (asset: string) => {
    toggle_market(asset);
    setMarkets(prev => prev.map(m => (m.asset === asset ? { ...m, is_paused: true } : m)));
    setMarketToToggle(null);
  };
  const handleRemove = (asset: string) => {
    remove_market(asset);
    setMarkets(prev => prev.filter(m => m.asset !== asset));
    setMarketToRemove(null);
  };
  const closeAll = async () => {
    await fetch('http://localhost:8090/command', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ closeAll: null }),
    });
  };
  const pauseAll = async () => {
    await fetch('http://localhost:8090/command', {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pauseAll: null }),
    });
  };

  return (
    <div className="relative min-h-screen overflow-hidden bg-[#07090B] text-white">
      {/* layered background */}
      <div className="pointer-events-none absolute inset-0 opacity-[0.08] [background:radial-gradient(60%_60%_at_0%_0%,rgba(56,189,248,0.5),transparent_60%),radial-gradient(50%_50%_at_100%_0%,rgba(232,121,249,0.5),transparent_60%),radial-gradient(60%_60%_at_50%_100%,rgba(52,211,153,0.4),transparent_60%)]" />
      <div className="pointer-events-none absolute inset-0 opacity-[0.06] bg-[linear-gradient(transparent_23px,rgba(255,255,255,0.06)_24px),linear-gradient(90deg,transparent_23px,rgba(255,255,255,0.06)_24px)] bg-[size:26px_26px]" />
      <div className="pointer-events-none absolute inset-0 [mask-image:linear-gradient(to_bottom,transparent,black_20%,black_80%,transparent)]">
        <div className="h-[200%] w-[200%] -translate-x-1/4 animate-[scan_9s_linear_infinite] bg-[repeating-linear-gradient(90deg,transparent_0,transparent_48px,rgba(255,255,255,0.04)_49px,rgba(255,255,255,0.04)_50px)]" />
      </div>

      <div className="mx-auto grid max-w-7xl grid-cols-1 gap-8 px-6 py-10 lg:grid-cols-[280px,1fr]">
        {/* Command Dock */}
        <aside className="h-fit rounded-md border border-white/10 bg-[#0B0E12]/80 p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]">
          <div className="flex items-baseline justify-between">
            <div>
              <div className="text-[10px] uppercase text-white/50">Available Margin</div>
              <div className="font-mono text-3xl tabular-nums tracking-tight">${totalMargin.toFixed(2)}</div>
            </div>
            <div className="h-6 w-1 bg-gradient-to-b from-cyan-400 via-fuchsia-400 to-emerald-400" />
          </div>

          <div className="mt-4 grid gap-2">
            {markets.length !== 0 && (
              <button onClick={() => setShowAdd(true)} className="w-full rounded-md border border-cyan-400/40 bg-cyan-500/10 px-3 py-2 text-cyan-200 hover:bg-cyan-500/20">
                <div className="flex items-center justify-center gap-2"><Plus className="h-4 w-4" /><span className="text-sm">Add Market</span></div>
              </button>
            )}
            <button
              className="w-full rounded-md border border-red-500/40 bg-red-600/15 px-3 py-2 text-red-200 hover:bg-red-600/25"
              onClick={() => { closeAll(); setMarkets([]); }}
            >
              <div className="flex items-center justify-center gap-2"><Power className="h-4 w-4" /><span className="text-sm">Close All</span></div>
            </button>
            <button
              className="w-full rounded-md border border-amber-500/40 bg-amber-500/15 px-3 py-2 text-amber-200 hover:bg-amber-500/25"
              onClick={() => { pauseAll(); markets.forEach(m => (m.is_paused = true)); }}
            >
              <div className="flex items-center justify-center gap-2"><Pause className="h-4 w-4" /><span className="text-sm">Pause All</span></div>
            </button>
          </div>

          <div className="mt-6 grid gap-2 border-t border-white/10 pt-4 text-[12px] text-white/60">
            <p className="font-semibold text-white/70">Console</p>
            <div className="rounded-md border border-white/10 bg-[#0F1115] p-3">
                
            </div>
          </div>
        </aside>

        {/* Markets Grid */}
        <main>
          {markets.length === 0 && (
            <div className="grid place-items-center rounded-md border border-white/10 bg-[#0B0E12]/80 p-12 text-center">
              <div>
                <h2 className="text-2xl font-semibold">No markets configured</h2>
                <p className="mt-1 text-white/60">Add a market to begin streaming quotes and executing strategies.</p>
                <button onClick={() => setShowAdd(true)} className="mt-5 inline-flex items-center gap-2 rounded-md border border-cyan-400/40 bg-cyan-500/10 px-4 py-2 text-cyan-200 hover:bg-cyan-500/20">
                  <Plus className="h-4 w-4" /> Add Market
                </button>
              </div>
            </div>
          )}

          {markets.length > 0 && (
            <div className="grid grid-cols-1 gap-7 sm:grid-cols-2 xl:grid-cols-3">
              {markets.map(m => (
                <motion.div key={m.asset} initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }}>
                  <MarketCard market={m} onTogglePause={() => handleConfirmToggle(m.asset, m.is_paused)} onRemove={() => setMarketToRemove(m.asset)} />
                </motion.div>
              ))}
            </div>
          )}
        </main>
      </div>

      {/* Error toast */}
      <AnimatePresence>
        {errorMsg && (
          <motion.div initial={{ y: -16, opacity: 0 }} animate={{ y: 0, opacity: 1 }} exit={{ y: -16, opacity: 0 }} className="fixed left-1/2 top-6 z-50 -translate-x-1/2">
            <div className="flex items-center gap-2 rounded-md border border-red-500/40 bg-[#2A1010] px-3 py-2 text-red-100 shadow">
              <AlertCircle className="h-4 w-4" />
              <span className="text-sm">{errorMsg}</span>
              <button onClick={() => setErrorMsg(null)} className="ml-2 rounded-md px-2 py-1 hover:bg-white/10"><X className="h-4 w-4" /></button>
            </div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Add Market modal */}
      <AnimatePresence>
        {showAdd && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="fixed inset-0 z-50">
            <div className="absolute inset-0 bg-black/70" onClick={() => setShowAdd(false)} />
            <motion.div initial={{ rotateX: -8, opacity: 0 }} animate={{ rotateX: 0, opacity: 1 }} exit={{ rotateX: -4, opacity: 0 }} className="relative mx-auto mt-24 w-full max-w-2xl rounded-md border border-white/10 bg-[#0B0E12] p-6">
              <div className="flex items-center justify-between border-b border-white/10 pb-3">
                <h3 className="text-sm text-white/80">Add Market</h3>
                <button onClick={() => setShowAdd(false)} className="rounded-md p-1 hover:bg-white/10"><X className="h-5 w-5" /></button>
              </div>
              <div className="pt-4">
                <AddMarket onClose={() => setShowAdd(false)} totalMargin={totalMargin} />
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Confirm remove */}
      <AnimatePresence>
        {marketToRemove && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="fixed inset-0 z-50">
            <div className="absolute inset-0 bg-black/70" onClick={() => setMarketToRemove(null)} />
            <motion.div initial={{ y: 24, opacity: 0 }} animate={{ y: 0, opacity: 1 }} exit={{ y: 10, opacity: 0 }} className="relative mx-auto mt-28 w-full max-w-md rounded-md border border-red-500/40 bg-[#1A0F12] p-6">
              <h3 className="text-lg font-semibold">Remove <span className="text-red-300">{marketToRemove}</span>?</h3>
              <p className="mt-1 text-red-200/80">This will close any ongoing trade initiated by the Bot.</p>
              <div className="mt-6 flex justify-end gap-2">
                <button className="rounded-md border border-white/20 px-4 py-2 hover:bg-white/10" onClick={() => setMarketToRemove(null)}>Cancel</button>
                <button className="rounded-md bg-red-600 px-4 py-2 text-white hover:bg-red-700" onClick={() => handleRemove(marketToRemove)}>Yes</button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      {/* Confirm pause */}
      <AnimatePresence>
        {marketToToggle && (
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="fixed inset-0 z-50">
            <div className="absolute inset-0 bg-black/70" onClick={() => setMarketToToggle(null)} />
            <motion.div initial={{ y: 24, opacity: 0 }} animate={{ y: 0, opacity: 1 }} exit={{ y: 10, opacity: 0 }} className="relative mx-auto mt-28 w-full max-w-md rounded-md border border-amber-500/40 bg-[#1A140A] p-6">
              <h3 className="text-lg font-semibold">Pause <span className="text-amber-300">{marketToToggle}</span>?</h3>
              <p className="mt-1 text-amber-200/80">This will close any ongoing trade initiated by the Bot.</p>
              <div className="mt-6 flex justify-end gap-2">
                <button className="rounded-md border border-white/20 px-4 py-2 hover:bg-white/10" onClick={() => setMarketToToggle(null)}>Cancel</button>
                <button className="rounded-md bg-amber-600 px-4 py-2 text-white hover:bg-amber-700" onClick={() => handleTogglePause(marketToToggle)}>Yes</button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      <style>{`@keyframes scan{0%{transform:translateX(0)}100%{transform:translateX(-25%)}}`}</style>
    </div>
  );
}
