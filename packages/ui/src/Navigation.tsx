'use client';

import React from 'react';
import Link from 'next/link';
import { Button } from "./button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { Wallet, ChevronDown, Shield, LogOut, User } from "lucide-react";
import { useWalletContext } from './wallet';
import { cn } from './lib/utils';

export interface NavigationItem {
  name: string;
  href: string;
}

interface NavigationProps {
  navItems?: NavigationItem[];
  activePath?: string;
}

export const Navigation = ({ navItems = [], activePath }: NavigationProps): React.ReactElement => {
  const { isConnected, accountId, isLoading, connect, disconnect } = useWalletContext();

  const defaultNavItems = [
    { name: "Dreamweave", href: "#dreamweave" },
    { name: "Groupweave", href: "#groupweave" },
    { name: "Company", href: "#company" },
  ];

  const navigationItems = navItems.length > 0 ? navItems : defaultNavItems;

  const handleConnect = async () => {
    try {
      await connect();
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    }
  };

  const handleDisconnect = async () => {
    try {
      await disconnect();
    } catch (error) {
      console.error('Failed to disconnect wallet:', error);
    }
  };

  const formatAccountId = (accountId: string) => {
    if (accountId.length > 20) {
      return `${accountId.slice(0, 8)}...${accountId.slice(-8)}`;
    }
    return accountId;
  };

  return (
    <nav
      className="fixed top-0 left-0 right-0 z-50 border-b transition-all duration-300 ease-in-out bg-white/80 dark:bg-zinc-900/80 backdrop-blur-xl border-slate-900/10 dark:border-slate-50/[0.06] shadow-sm"
    >
      <div className="max-w-7xl mx-auto px-6 lg:px-8">
        <div className="flex items-center justify-between h-20">
          {/* Logo/Brand */}
          <Link href="/" className="flex items-center">
            <div className="flex items-center gap-3">
              {/* Premium Logo */}
              <div className="relative group">
                <div className="w-10 h-10 rounded-xl shadow-lg flex items-center justify-center transition-all duration-300 group-hover:scale-110 group-hover:shadow-xl group-hover:rotate-3 bg-gradient-to-br from-purple-500 to-indigo-600">
                  <div className="w-6 h-6 bg-white rounded-md transition-all duration-300 group-hover:bg-opacity-90"></div>
                </div>
                <div className="absolute -top-1 -right-1 w-4 h-4 bg-blue-500 rounded-full shadow-md transition-all duration-300 group-hover:scale-125 group-hover:animate-pulse"></div>
              </div>

              {/* Brand Typography */}
              <div className="flex flex-col group">
                <span className="text-xl font-bold text-gray-900 dark:text-gray-100 tracking-tight leading-none transition-all duration-300 group-hover:text-blue-600 dark:group-hover:text-blue-400 group-hover:scale-105">
                  Torus
                </span>
                <span className="text-xs text-gray-500 dark:text-gray-400 font-mono tracking-wide transition-all duration-300 group-hover:text-gray-700 dark:group-hover:text-gray-300 group-hover:tracking-wider">
                  GroupWeave
                </span>
              </div>
            </div>
          </Link>

          {/* Center Navigation Links */}
          <div className="hidden lg:flex items-center space-x-1">
            {navigationItems.map((item) => (
              <Link href={item.href} key={item.name} passHref>
                <Button
                  variant="minimal"
                  className={cn(
                    "text-sm font-medium px-4 py-2 h-10 transition-all duration-300 hover:scale-105 hover:bg-gray-100 dark:hover:bg-gray-800 hover:shadow-md relative overflow-hidden group",
                    activePath === item.href && "bg-slate-100 dark:bg-slate-800 shadow-inner"
                  )}
                >
                  <span className="relative z-10 transition-colors duration-300">{item.name}</span>
                  <div className="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 opacity-0 group-hover:opacity-10 transition-opacity duration-300"></div>
                </Button>
              </Link>
            ))}
          </div>

          {/* Right Side Actions */}
          <div className="flex items-center space-x-4">
            {/* Security Badge */}
            <div className="hidden md:flex items-center gap-2 px-3 py-1.5 rounded-full border border-green-200 dark:border-green-700 bg-green-50 dark:bg-green-900/50 transition-all duration-300 hover:shadow-md hover:scale-105 group cursor-pointer">
              <Shield className="w-4 h-4 text-green-600 dark:text-green-400 transition-transform duration-300 group-hover:rotate-12" />
              <span className="text-xs font-medium text-green-700 dark:text-green-300 transition-colors duration-300 group-hover:text-green-800 dark:group-hover:text-green-200">Secured</span>
            </div>

            {/* Wallet Connect/Account Dropdown */}
            {!isConnected ? (
              <Button
                variant="wallet"
                className="relative overflow-hidden group transition-all duration-300 hover:scale-105 hover:shadow-lg"
                onClick={handleConnect}
                disabled={isLoading}
              >
                <div className="flex items-center gap-2 relative z-10">
                  <div className="p-1.5 rounded-md bg-blue-100 dark:bg-blue-900/50 group-hover:bg-blue-200 dark:group-hover:bg-blue-800/50 transition-colors duration-300">
                    <Wallet className="w-4 h-4 text-blue-600 dark:text-blue-400" />
                  </div>
                  <span className="font-medium text-gray-700 dark:text-gray-200 group-hover:text-gray-900 dark:group-hover:text-white transition-colors duration-300">
                    {isLoading ? 'Connecting...' : 'Connect Wallet'}
                  </span>
                </div>
                <div className="absolute inset-0 bg-gradient-to-r from-blue-500 to-purple-600 opacity-0 group-hover:opacity-5 transition-opacity duration-300"></div>
              </Button>
            ) : (
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="wallet"
                    className="relative overflow-hidden group transition-all duration-300 hover:scale-105 hover:shadow-lg"
                  >
                    <div className="flex items-center gap-2 relative z-10">
                      <div className="p-1.5 rounded-md bg-green-100 dark:bg-green-900/50 group-hover:bg-green-200 dark:group-hover:bg-green-800/50 transition-colors duration-300">
                        <User className="w-4 h-4 text-green-600 dark:text-green-400" />
                      </div>
                      <span className="font-medium text-gray-700 dark:text-gray-200 group-hover:text-gray-900 dark:group-hover:text-white transition-colors duration-300">
                        {accountId ? formatAccountId(accountId) : 'Connected'}
                      </span>
                      <ChevronDown className="w-4 h-4 text-gray-500 dark:text-gray-400 group-hover:text-gray-700 dark:group-hover:text-gray-300 transition-all duration-300 group-hover:rotate-180" />
                    </div>
                    <div className="absolute inset-0 bg-gradient-to-r from-green-500 to-blue-600 opacity-0 group-hover:opacity-5 transition-opacity duration-300"></div>
                  </Button>
                </DropdownMenuTrigger>

                <DropdownMenuContent
                  align="end"
                  className="w-64 p-4 mt-2 border rounded-xl shadow-xl bg-white/95 dark:bg-zinc-900/95 backdrop-blur-xl border-slate-900/10 dark:border-slate-50/[0.06]"
                >
                  {/* Account Info */}
                  <div className="mb-4 pb-3 border-b border-gray-100 dark:border-gray-800">
                    <h3 className="font-semibold text-gray-900 dark:text-gray-100 mb-1">Connected Account</h3>
                    <p className="text-sm text-gray-600 dark:text-gray-400 font-mono break-all">{accountId}</p>
                  </div>

                  {/* Disconnect Button */}
                  <DropdownMenuItem
                    className="p-0 focus:bg-transparent"
                    onClick={handleDisconnect}
                  >
                    <div className="w-full p-3 rounded-lg border border-red-200/50 dark:border-red-500/30 hover:border-red-200 dark:hover:border-red-500/50 hover:bg-red-50 dark:hover:bg-red-900/20 transition-all duration-300 cursor-pointer group">
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 rounded-lg bg-red-100 dark:bg-red-900/30 flex items-center justify-center transition-transform duration-300 group-hover:scale-110">
                          <LogOut className="w-4 h-4 text-red-600 dark:text-red-400" />
                        </div>
                        <span className="font-medium text-red-700 dark:text-red-300 group-hover:text-red-800 dark:group-hover:text-red-200 transition-colors duration-300">
                          Disconnect Wallet
                        </span>
                      </div>
                    </div>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            )}
          </div>
        </div>
      </div>
    </nav>
  );
};