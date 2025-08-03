'use client';

import { Button } from "./button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from "./ui/dropdown-menu";
import { Wallet, ChevronDown, Shield, ArrowUpRight } from "lucide-react";

export const Navigation = () => {
  const walletOptions = [
    { 
      name: "NEAR Protocol", 
      tag: "Recommended",
      icon: "◼", 
      color: "text-crypto-near",
      bgColor: "bg-crypto-near/10",
      description: "Fast & low-cost transactions"
    },
    { 
      name: "Ethereum Wallets", 
      icon: "♦", 
      color: "text-crypto-ethereum",
      bgColor: "bg-crypto-ethereum/10",
      description: "MetaMask, Coinbase, WalletConnect"
    },
    { 
      name: "MetaMask", 
      icon: "🦊", 
      color: "text-crypto-metamask",
      bgColor: "bg-crypto-metamask/10",
      description: "Most popular browser wallet"
    },
    { 
      name: "WalletConnect", 
      icon: "🔗", 
      color: "text-crypto-walletconnect",
      bgColor: "bg-crypto-walletconnect/10",
      description: "Connect mobile wallets"
    },
    { 
      name: "Coinbase Wallet", 
      icon: "🔵", 
      color: "text-crypto-coinbase",
      bgColor: "bg-crypto-coinbase/10",
      description: "Secure & user-friendly"
    },
  ];

  const navItems = [
    { name: "Products", href: "#products" },
    { name: "Solutions", href: "#solutions" },
    { name: "Developers", href: "#developers" },
    { name: "Company", href: "#company" },
  ];

  const handleWalletConnect = (walletName: string) => {
    console.log(`Connecting to ${walletName}...`);
    // Add your wallet connection logic here
  };

  return (
    <nav className="fixed top-0 left-0 right-0 z-50 nav-glass border-b border-nav-border/50">
      <div className="max-w-7xl mx-auto px-6 lg:px-8">
        <div className="flex items-center justify-between h-20">
          {/* Logo/Brand */}
          <div className="flex items-center">
            <div className="flex items-center gap-3">
              {/* Premium Logo */}
              <div className="relative">
                <div className="w-10 h-10 logo-gradient rounded-xl shadow-lg flex items-center justify-center">
                  <div className="w-6 h-6 bg-white rounded-md"></div>
                </div>
                <div className="absolute -top-1 -right-1 w-4 h-4 bg-accent rounded-full shadow-md"></div>
              </div>
              
              {/* Brand Typography */}
              <div className="flex flex-col">
                <span className="text-xl font-bold text-text-primary tracking-tight leading-none">
                  Orb
                </span>
                <span className="text-xs text-text-tertiary font-mono tracking-wide">
                  PROTOCOL
                </span>
              </div>
            </div>
          </div>

          {/* Center Navigation Links */}
          <div className="hidden lg:flex items-center space-x-1">
            {navItems.map((item) => (
              <Button 
                key={item.name}
                variant="minimal" 
                className="text-sm font-medium px-4 py-2 h-10"
              >
                {item.name}
              </Button>
            ))}
          </div>

          {/* Right Side Actions */}
          <div className="flex items-center gap-4">
            {/* Security Badge */}
            <div className="hidden md:flex items-center gap-2 px-3 py-2 bg-surface-secondary rounded-lg">
              <Shield className="w-4 h-4 text-success" />
              <span className="text-xs font-medium text-text-secondary">Secured</span>
            </div>

            {/* Wallet Connect Dropdown */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button 
                  variant="wallet" 
                  size="lg"
                  className="flex items-center gap-3 relative overflow-hidden group"
                >
                  {/* Animated background */}
                  <div className="absolute inset-0 bg-gradient-accent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
                  
                  <div className="relative flex items-center gap-3">
                    <Wallet className="w-5 h-5" />
                    <span className="font-semibold">Connect Wallet</span>
                    <ChevronDown className="w-4 h-4 group-hover:rotate-180 transition-transform duration-200" />
                  </div>
                </Button>
              </DropdownMenuTrigger>
              
              <DropdownMenuContent 
                align="end" 
                className="w-80 mt-3 bg-surface-primary/95 backdrop-blur-xl border border-nav-border shadow-2xl rounded-xl p-2"
                sideOffset={5}
              >
                {/* Header */}
                <div className="px-4 py-3 border-b border-nav-border/50">
                  <h3 className="font-semibold text-text-primary">Choose Wallet</h3>
                  <p className="text-sm text-text-tertiary mt-1">Connect your crypto wallet to continue</p>
                </div>

                {/* Wallet Options */}
                <div className="py-2">
                  {walletOptions.map((wallet, index) => (
                    <DropdownMenuItem
                      key={wallet.name}
                      onClick={() => handleWalletConnect(wallet.name)}
                      className="wallet-item cursor-pointer focus:bg-surface-secondary data-[highlighted]:bg-surface-secondary m-1 rounded-lg group"
                    >
                      <div className="flex items-center gap-4 w-full">
                        {/* Wallet Icon */}
                        <div className={`w-12 h-12 ${wallet.bgColor} rounded-xl flex items-center justify-center text-lg group-hover:scale-110 transition-transform duration-200`}>
                          {wallet.icon}
                        </div>
                        
                        {/* Wallet Info */}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="font-semibold text-text-primary">{wallet.name}</span>
                            {wallet.tag && (
                              <span className="px-2 py-1 bg-accent/10 text-accent text-xs font-medium rounded-md">
                                {wallet.tag}
                              </span>
                            )}
                          </div>
                          <p className="text-sm text-text-tertiary mt-1">{wallet.description}</p>
                        </div>
                        
                        {/* Connect Arrow */}
                        <ArrowUpRight className="w-5 h-5 text-text-quaternary group-hover:text-primary group-hover:translate-x-1 group-hover:-translate-y-1 transition-all duration-200" />
                      </div>
                    </DropdownMenuItem>
                  ))}
                </div>

                <DropdownMenuSeparator className="my-2" />
                
                {/* Footer */}
                <div className="px-4 py-3">
                  <p className="text-xs text-text-tertiary text-center">
                    New to crypto? <span className="text-primary font-medium cursor-pointer hover:underline">Learn more</span>
                  </p>
                </div>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>
    </nav>
  );
};