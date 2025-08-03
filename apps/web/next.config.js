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
  webpack: (config, { isServer }) => {
    // Polyfills for browser environment
    if (!isServer) {
      config.resolve.fallback = {
        net: false,
        fs: false,
        tls: false,
        crypto: false,
        stream: 'stream-browserify',
        url: 'url',
        http: 'stream-http',
        https: 'https-browserify',
        assert: 'assert',
        os: 'os-browserify',
        path: 'path-browserify',
      };
    }

    return config;
  },
};

export default nextConfig;