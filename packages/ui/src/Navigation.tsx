'use client';

import React from 'react';
import { Button } from "./button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { Wallet, ChevronDown, Shield, LogOut } from "lucide-react";
import { navItems } from './config/navigation';
import { Logo } from './Logo';

interface NavigationProps {
  className?: string;
  accountId: string | null;
  onConnect: () => void;
  onDisconnect: () => void;
}

export const Navigation = ({ className, accountId, onConnect, onDisconnect }: NavigationProps): React.ReactElement => {
  const formatAccountId = (id: string) => {
    if (id.length > 20) {
      return `${id.slice(0, 10)}...${id.slice(-4)}`;
    }
    return id;
  };

  return (
    <nav 
      className="fixed top-0 left-0 right-0 z-50 border-b transition-all duration-300 ease-in-out bg-white/80 backdrop-blur-[12px] border-black/10 shadow-sm"
    >
      <div className="max-w-7xl mx-auto px-6 lg:px-8">
        <div className="flex items-center justify-between h-20">
          {/* Logo/Brand */}
          <div className="flex items-center">
            <div className="flex items-center gap-3">
              {/* Premium Logo */}
              <Logo className="w-10 h-10" />
              
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
            {navItems.map((item) => (
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

            {/* Wallet Connect Button / Dropdown */}
            {accountId ? (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="wallet"
                    className="relative overflow-hidden group transition-all duration-300 hover:scale-105 hover:shadow-lg"
                  >
                    <div className="flex items-center gap-2 relative z-10">
                      <div className="p-1.5 rounded-md bg-green-100 group-hover:bg-green-200 transition-colors duration-300">
                        <Wallet className="w-4 h-4 text-green-600" />
                      </div>
                      <span className="font-medium text-gray-700 group-hover:text-gray-900 transition-colors duration-300">{formatAccountId(accountId)}</span>
                      <ChevronDown className="w-4 h-4 text-gray-500 group-hover:text-gray-700 transition-all duration-300 group-hover:rotate-180" />
                    </div>
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-56 mt-2">
                  <DropdownMenuItem onClick={onDisconnect} className="cursor-pointer">
                    <LogOut className="mr-2 h-4 w-4" />
                    <span>Disconnect</span>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            ) : (
              <Button
                variant="wallet"
                className="relative overflow-hidden group transition-all duration-300 hover:scale-105 hover:shadow-lg"
                onClick={onConnect}
              >
                <div className="flex items-center gap-2 relative z-10">
                  <div className="p-1.5 rounded-md bg-blue-100 group-hover:bg-blue-200 transition-colors duration-300">
                    <Wallet className="w-4 h-4 text-blue-600" />
                  </div>
                  <span className="font-medium text-gray-700 group-hover:text-gray-900 transition-colors duration-300">Connect Wallet</span>
                </div>
              </Button>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
};