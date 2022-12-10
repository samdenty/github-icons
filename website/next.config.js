const relay = require('./relay.config');

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  swcMinify: true,
  compiler: {
    emotion: true,
    relay,
  },

  webpack: (config, { isServer, webpack }) => {
    if (!isServer) {
      // Ensures no server modules are included on the client.
      config.plugins.push(
        new webpack.IgnorePlugin({
          resourceRegExp: /lib\/server/,
        })
      );
    }

    return config;
  },
};

module.exports = nextConfig;
