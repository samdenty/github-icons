import webpack from 'webpack';
import path from 'path';
import CopyPlugin from 'copy-webpack-plugin';

const src = path.join(__dirname, 'src');

const config: webpack.Configuration = {
  entry: {
    options: path.join(src, 'options.tsx'),
    background: path.join(src, 'background.ts'),
    content_script: path.join(src, 'content_script/index.ts'),
  },
  devtool: 'inline-source-map',
  output: {
    path: path.join(__dirname, 'dist'),
    filename: '[name].js',
  },
  optimization: {
    splitChunks: {
      name: 'vendor',
      chunks(chunk) {
        return chunk.name !== 'background';
      },
    },
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: ['.ts', '.tsx', '.js'],
  },
  plugins: [
    new CopyPlugin({
      patterns: [{ from: '.', to: '.', context: 'public' }],
      options: {},
    }),
  ],
};

export default config;
