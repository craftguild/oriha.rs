const path = require('path');
const { getDefaultConfig } = require('expo/metro-config');

const projectRoot = __dirname;
const workspaceRoot = path.resolve(projectRoot, '../..');

const config = getDefaultConfig(projectRoot);

config.watchFolders = [
  ...(config.watchFolders ?? []),
  path.join(workspaceRoot, 'packages/expo-oriha'),
];
config.resolver.sourceExts = [...new Set([...config.resolver.sourceExts, 'cjs'])];
config.resolver.nodeModulesPaths = [
  path.join(projectRoot, 'node_modules'),
  path.join(workspaceRoot, 'node_modules'),
];

config.resolver.extraNodeModules = {
  ...(config.resolver.extraNodeModules ?? {}),
  '@craftguild/expo-oriha': path.join(workspaceRoot, 'packages/expo-oriha'),
  expo: path.join(projectRoot, 'node_modules/expo'),
  'expo-crypto': path.join(projectRoot, 'node_modules/expo-crypto'),
  'expo-file-system': path.join(projectRoot, 'node_modules/expo-file-system'),
  'expo-modules-core': path.join(projectRoot, 'node_modules/expo-modules-core'),
  'expo-print': path.join(projectRoot, 'node_modules/expo-print'),
  'expo-sharing': path.join(projectRoot, 'node_modules/expo-sharing'),
  react: path.join(projectRoot, 'node_modules/react'),
  'react-native': path.join(projectRoot, 'node_modules/react-native'),
  'react-native-svg': path.join(projectRoot, 'node_modules/react-native-svg'),
  'react-native-webview': path.join(projectRoot, 'node_modules/react-native-webview'),
};

const defaultResolveRequest = config.resolver.resolveRequest;
const sharedModules = config.resolver.extraNodeModules;
config.resolver.resolveRequest = (context, moduleName, platform) => {
  const sharedName = Object.keys(sharedModules).find(
    (name) => moduleName === name || moduleName.startsWith(`${name}/`),
  );
  const request = sharedName
    ? path.join(sharedModules[sharedName], moduleName.slice(sharedName.length))
    : moduleName;
  return defaultResolveRequest
    ? defaultResolveRequest(context, request, platform)
    : context.resolveRequest(context, request, platform);
};

module.exports = config;
