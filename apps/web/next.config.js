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
  webpack: (config) => {
    config.resolve.fallback = {
      ...config.resolve.fallback,
      fs: false,
      net: false,
      tls: false,
      crypto: require.resolve('crypto-browserify'),
      stream: require.resolve('stream-browserify'),
      http: require.resolve('stream-http'),
      https: require.resolve('https-browserify'),
      zlib: require.resolve('browserify-zlib'),
      url: require.resolve('url/'),
      assert: require.resolve('assert/'),
    };
    return config;
  },
};

export default nextConfig;