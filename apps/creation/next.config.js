/** @type {import('next').NextConfig} */
const nextConfig = {
  webpack: (config, { isServer, webpack }) => {
    if (!isServer) {
      // Provide polyfills for Node.js globals
      config.resolve.fallback = {
        ...config.resolve.fallback,
        fs: false,
        path: false,
        os: false,
        crypto: 'crypto-browserify',
        stream: 'stream-browserify',
        buffer: 'buffer',
        util: 'util',
        url: false,
        querystring: false,
        net: false,
        tls: false,
        child_process: false,
        readline: false,
        zlib: false,
        http: false,
        https: false,
        assert: false,
        constants: false,
        timers: false,
        console: false,
        vm: false,
        process: 'process/browser',
      };

      // Exclude problematic Node.js modules from client bundle
      config.externals = config.externals || [];
      config.externals.push({
        '@near-js/keystores-node': 'commonjs @near-js/keystores-node',
      });

      // Add global polyfills
      config.plugins = config.plugins || [];
      config.plugins.push(
        new webpack.ProvidePlugin({
          Buffer: ['buffer', 'Buffer'],
          process: 'process/browser',
        })
      );
    }



    return config;
  },
};

export default nextConfig;
