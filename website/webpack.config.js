const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const { CleanWebpackPlugin } = require('clean-webpack-plugin');

module.exports = {
  // Entry point of your application
  entry: './src/index.ts', // Ensure this path is correct

  // Output configuration
  output: {
    filename: '[name].bundle.js',
    path: path.resolve(__dirname, 'dist'),
    publicPath: '/', 
    chunkFilename: '[name].bundle.js',
  },

  // Resolve extensions so you can import without specifying them
  resolve: {
    extensions: ['.ts', '.js'], // Make sure Webpack looks for TypeScript and JavaScript files
  },

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
      // Images and assets loader
      {
        test: /\.(png|jpe?g|gif)$/i,
        type: 'asset/resource',
      },
    ],
  },

  plugins: [
    new CleanWebpackPlugin(),
    new HtmlWebpackPlugin({
      template: './public/index.html',
      filename: 'index.html',
    }),
  ],

  devServer: {
    static: {
      directory: path.join(__dirname, 'dist'),
    },
    compress: true,
    port: 9000,
    open: true,
    hot: true,
    historyApiFallback: true,
  },

  devtool: 'source-map',

  optimization: {
    splitChunks: {
      chunks: 'all',
    },
  },
};

