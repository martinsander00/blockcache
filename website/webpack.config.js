// webpack.config.js

const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');

module.exports = {
  // Entry point of your application
  entry: './src/index.ts',

  // Output configuration
  output: {
    filename: '[name].bundle.js', // Use dynamic names for entry chunks
    path: path.resolve(__dirname, 'dist'),
    publicPath: '/', // Necessary for dev server routing
    chunkFilename: '[name].bundle.js', // Use dynamic names for additional chunks
  },

  // Resolve extensions so you can import without specifying them
  resolve: {
    extensions: ['.ts', '.js'],
  },

  // Module rules to handle different file types
  module: {
    rules: [
      // TypeScript loader
      {
        test: /\.ts$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      // CSS loaders
      {
        test: /\.css$/i,
        use: ['style-loader', 'css-loader'],
      },
      // Source map loader
      {
        enforce: 'pre',
        test: /\.js$/,
        loader: 'source-map-loader',
      },
    ],
  },

  // Plugins
  plugins: [
    new CleanWebpackPlugin(), // Cleans the dist folder before each build
    new HtmlWebpackPlugin({
      template: './public/index.html', // Template file
      filename: 'index.html', // Output file
    }),
  ],

  // Development server configuration
  devServer: {
    static: {
      directory: path.join(__dirname, 'dist'), // Serve from 'dist' directory
    },
    compress: true, // Enable gzip compression
    port: 9000, // Port number
    open: true, // Open browser on server start
    hot: true, // Enable hot module replacement
    historyApiFallback: true, // For single-page applications
  },

  // Source maps for easier debugging
  devtool: 'source-map',

  // Optimization settings
  optimization: {
    splitChunks: {
      chunks: 'all', // Split vendor and commons
    },
  },
};

