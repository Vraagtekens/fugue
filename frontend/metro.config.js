// metro.config.js
// const { getDefaultConfig } = require('expo/metro-config');
// const { withNativeWind } = require('nativewind/metro');

// const config = getDefaultConfig(__dirname);

// module.exports = withNativeWind(config, { input: './global.css', inlineRem: 16 });

// metro.config.js
// const { getDefaultConfig } = require('expo/metro-config');
// const { withNativeWind } = require('nativewind/metro');

// const config = getDefaultConfig(__dirname);

// // Add .glb and .gltf to assetExts so Metro knows they are static assets
// config.resolver.assetExts.push('glb', 'gltf');

// module.exports = withNativeWind(config, { input: './global.css', inlineRem: 16 });

// metro.config.js
const { getDefaultConfig } = require('expo/metro-config');
const { withNativeWind } = require('nativewind/metro');
const path = require('path');

const config = getDefaultConfig(__dirname);

// Ensure Three.js is resolved as a single instance
config.resolver.extraNodeModules = {
  ...(config.resolver.extraNodeModules || {}),
  three: path.resolve(__dirname, 'node_modules/three'),
};

// Add .glb and .gltf to assetExts
config.resolver.assetExts.push('glb', 'gltf');

module.exports = withNativeWind(config, {
  input: './global.css',
  inlineRem: 16,
});
