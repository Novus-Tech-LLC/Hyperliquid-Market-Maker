import React from 'react';
import { Github, ExternalLink } from 'lucide-react';

const Header: React.FC = () => (
  <header className="sticky top-0 z-40 border-b border-white/10 bg-[#0B0C0E]">
    <div className="mx-auto flex max-w-7xl items-center justify-between px-6 py-3">
      <div className="flex items-center gap-3">
        <div className="grid h-8 w-8 place-items-center rounded-md border border-white/10 bg-[#111316]">
          <div className="h-3.5 w-3.5 bg-orange-500" />
        </div>
        <div className="leading-tight">
          <h1 className="font-mono text-sm tracking-[0.18em] text-white">KWANT</h1>
          <p className="text-[10px] uppercase text-white/50">Trading Bot Console</p>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <a
          href="https://app.hyperliquid.xyz"
          target="_blank"
          rel="noopener noreferrer"
          className="hidden md:inline-flex items-center gap-2 rounded-md border border-white/10 bg-[#111316] px-3 py-1 text-[12px] text-white hover:bg-white/5"
        >
          <ExternalLink className="h-3.5 w-3.5 text-orange-400" /> Hyperliquid
        </a>
        <a
          href="https://github.com/0xNoSystem/hyperliquid_rust_bot"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2 rounded-md border border-white/10 bg-[#111316] px-3 py-1 text-white hover:bg-white/5"
        >
          <Github className="h-4 w-4" /> <span className="text-[12px]">Repo</span>
        </a>
      </div>
    </div>
  </header>
);

export default Header;
