import React from 'react';

export const Logo = ({ className }: { className?: string }) => (
  <svg
    className={className}
    width="40"
    height="40"
    viewBox="0 0 40 40"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
  >
    <rect width="40" height="40" rx="8" fill="url(#paint0_linear_1_2)" />
    <rect x="12" y="12" width="16" height="16" rx="4" fill="white" />
    <defs>
      <linearGradient
        id="paint0_linear_1_2"
        x1="0"
        y1="0"
        x2="40"
        y2="40"
        gradientUnits="userSpaceOnUse"
      >
        <stop stopColor="#667eea" />
        <stop offset="1" stopColor="#764ba2" />
      </linearGradient>
    </defs>
  </svg>
);
