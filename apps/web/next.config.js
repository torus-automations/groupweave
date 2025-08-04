/** @type {import('next').NextConfig} */
const nextConfig = {
  transpilePackages: [
    '@repo/ui',
    'chainsig.js',
    '@cosmjs/proto-signing',
    'cosmjs-types',
    '@near-js/keystores',
    '@near-js/crypto',
    '@near-js/utils',
    '@near-js/types',
  ],
  output: 'standalone',
};

export default nextConfig;