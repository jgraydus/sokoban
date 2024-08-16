const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const dist = path.resolve(__dirname, "dist");

module.exports = {
  mode: "production",
  entry: { index: "./static/index.js" },
  output: {
    path: dist,
    filename: "[name].js"
  },
  devServer: {
    watchFiles: {
      options: {
        ignored: path.resolve(__dirname, "**/*.swp"),
      }
    }
  },
  experiments: { asyncWebAssembly: true },
  plugins: [
    new CopyPlugin({
      patterns: [
        {
	  from: path.resolve(__dirname, "static"),
	  to: path.resolve(__dirname, "dist"),
    	}
      ]
    }),
    new WasmPackPlugin({ crateDirectory: __dirname })
  ]
}

