'use client';

import React from 'react';
import { Button } from "./button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from "./ui/dropdown-menu";
import { Wallet, ChevronDown, Shield, ArrowUpRight } from "lucide-react";

export interface NavigationItem {
  name: string;
  href: string;
}

interface NavigationProps {
  className?: string;
  navItems?: NavigationItem[];
}

export const Navigation = ({ className, navItems = [] }: NavigationProps): React.ReactElement => {
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

  // Default navigation items if none provided
  const defaultNavItems = [
    { name: "Dreamweave", href: "#dreamweave" },
    { name: "Groupweave", href: "#groupweave" },
    { name: "Company", href: "#company" },
  ];

  const navigationItems = navItems.length > 0 ? navItems : defaultNavItems;

  const handleWalletConnect = (walletName: string) => {
    console.log(`Connecting to ${walletName}...`);
    // Add your wallet connection logic here
  };

  return (
    <nav 
      className="fixed top-0 left-0 right-0 z-50 border-b transition-all duration-300 ease-in-out"
      style={{
        backgroundColor: 'rgba(255, 255, 255, 0.8)',
        backdropFilter: 'blur(12px)',
        borderColor: 'rgba(0, 0, 0, 0.1)',
        boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)'
      }}
    >
      <div className="max-w-7xl mx-auto px-6 lg:px-8">
        <div className="flex items-center justify-between h-20">
          {/* Logo/Brand */}
          <div className="flex items-center">
            <div className="flex items-center gap-3">
              {/* Premium Logo */}
              <div className="relative group cursor-pointer">
                <div className="w-10 h-10 rounded-xl shadow-lg flex items-center justify-center transition-all duration-300 group-hover:scale-110 group-hover:shadow-xl group-hover:rotate-3" style={{background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)'}}>
                  <div className="w-6 h-6 bg-white rounded-md transition-all duration-300 group-hover:bg-opacity-90"></div>
                </div>
                <div className="absolute -top-1 -right-1 w-4 h-4 bg-blue-500 rounded-full shadow-md transition-all duration-300 group-hover:scale-125 group-hover:animate-pulse"></div>
              </div>
              
              {/* Brand Typography */}
              <div className="flex flex-col group cursor-pointer">
                <span className="text-xl font-bold text-gray-900 tracking-tight leading-none transition-all duration-300 group-hover:text-blue-600 group-hover:scale-105">
                  Torus
                </span>
                <span className="text-xs text-gray-500 font-mono tracking-wide transition-all duration-300 group-hover:text-gray-700 group-hover:tracking-wider">
                  GroupWeave
                </span>
              </div>
            </div>
          </div>

          {/* Center Navigation Links */}
          <div className="hidden lg:flex items-center space-x-1">
            {navigationItems.map((item) => (
              <Button 
                key={item.name}
                variant="minimal" 
                className="text-sm font-medium px-4 py-2 h-10 transition-all duration-300 hover:scale-105 hover:bg-gray-100 hover:shadow-md relative overflow-hidden group"
              >
                <span className="relative z-10 transition-colors duration-300">{item.name}</span>
                <div className="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 opacity-0 group-hover:opacity-10 transition-opacity duration-300"></div>
              </Button>
            ))}
          </div>

          {/* Right Side Actions */}
          <div className="flex items-center space-x-4">
            {/* Security Badge */}
            <div className="hidden md:flex items-center gap-2 px-3 py-1.5 rounded-full border border-green-200 bg-green-50 transition-all duration-300 hover:shadow-md hover:scale-105 group cursor-pointer">
              <Shield className="w-4 h-4 text-green-600 transition-transform duration-300 group-hover:rotate-12" />
              <span className="text-xs font-medium text-green-700 transition-colors duration-300 group-hover:text-green-800">Secured</span>
            </div>

            {/* Wallet Connect Dropdown */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button 
                  variant="wallet" 
                  className="relative overflow-hidden group transition-all duration-300 hover:scale-105 hover:shadow-lg"
                >
                  <div className="flex items-center gap-2 relative z-10">
                    <div className="p-1.5 rounded-md bg-blue-100 group-hover:bg-blue-200 transition-colors duration-300">
                      <Wallet className="w-4 h-4 text-blue-600" />
                    </div>
                    <span className="font-medium text-gray-700 group-hover:text-gray-900 transition-colors duration-300">Connect</span>
                    <ChevronDown className="w-4 h-4 text-gray-500 group-hover:text-gray-700 transition-all duration-300 group-hover:rotate-180" />
                  </div>
                  <div className="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 opacity-0 group-hover:opacity-5 transition-opacity duration-300"></div>
                </Button>
              </DropdownMenuTrigger>
              
              <DropdownMenuContent 
                align="end" 
                className="w-80 p-4 mt-2 border border-gray-200 bg-white rounded-xl shadow-xl"
                style={{backdropFilter: 'blur(12px)', backgroundColor: 'rgba(255, 255, 255, 0.95)'}}
              >
                {/* Header */}
                <div className="mb-4 pb-3 border-b border-gray-100">
                  <h3 className="font-semibold text-gray-900 mb-1">Connect Wallet</h3>
                  <p className="text-sm text-gray-600">Choose your preferred wallet to get started</p>
                </div>
                
                {/* Wallet Options */}
                <div className="space-y-2 mb-4">
                  {walletOptions.map((wallet, index) => (
                    <DropdownMenuItem 
                      key={index}
                      className="p-0 focus:bg-transparent"
                      onClick={() => handleWalletConnect(wallet.name)}
                    >
                      <div className="w-full p-3 rounded-lg border border-gray-100 hover:border-blue-200 hover:bg-blue-50 transition-all duration-300 cursor-pointer group">
                        <div className="flex items-center gap-3">
                          <div className={`w-10 h-10 rounded-lg ${wallet.bgColor || 'bg-gray-100'} flex items-center justify-center text-lg font-bold transition-transform duration-300 group-hover:scale-110`}>
                            {wallet.icon}
                          </div>
                          <div className="flex-1">
                            <div className="flex items-center gap-2">
                              <span className="font-medium text-gray-900 group-hover:text-blue-700 transition-colors duration-300">{wallet.name}</span>
                              {wallet.tag && (
                                <span className="px-2 py-0.5 text-xs font-medium bg-blue-100 text-blue-700 rounded-full">{wallet.tag}</span>
                              )}
                            </div>
                            <p className="text-sm text-gray-600 mt-0.5">{wallet.description}</p>
                          </div>
                          <ArrowUpRight className="w-4 h-4 text-gray-400 group-hover:text-blue-600 transition-all duration-300 group-hover:translate-x-0.5 group-hover:-translate-y-0.5" />
                        </div>
                      </div>
                    </DropdownMenuItem>
                  ))}
                </div>
                
                <DropdownMenuSeparator className="my-3" />
                
                {/* Footer */}
                <div className="text-center pt-2">
                  <p className="text-xs text-gray-500">
                    New to crypto? <span className="text-blue-600 hover:text-blue-700 cursor-pointer font-medium">Learn more</span>
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