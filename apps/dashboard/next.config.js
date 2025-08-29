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

      // Add global polyfills
      config.plugins = config.plugins || [];
      config.plugins.push(
        new webpack.ProvidePlugin({
          Buffer: ['buffer', 'Buffer'],
          process: 'process/browser',
        })
      );
    }
    
    // Ignore Node.js specific modules and problematic packages in client bundles
    config.externals = config.externals || [];
    if (!isServer) {
      config.externals.push({
        'fs': 'commonjs fs',
        'path': 'commonjs path',
        'os': 'commonjs os',
        'pino-pretty': 'pino-pretty',
        '@near-js/keystores-node': 'commonjs @near-js/keystores-node',
        'near-api-js/lib/key_stores/unencrypted_file_system_keystore': 'commonjs near-api-js/lib/key_stores/unencrypted_file_system_keystore',
      });
    }
    
    // Add module resolution aliases to avoid Node.js modules
    config.resolve.alias = {
      ...config.resolve.alias,
      '@near-js/keystores-node': false,
    };
    
    return config;
  },
};

export default nextConfig;
