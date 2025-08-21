import path from 'path';
import { fileURLToPath } from 'url';
import ForkTsCheckerWebpackPlugin from 'fork-ts-checker-webpack-plugin';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default {
  webpack: {
    configure: (webpackConfig) => {
      // Remove ModuleScopePlugin to allow imports outside src/
      const moduleScopePluginIndex = webpackConfig.resolve.plugins.findIndex(
        (plugin) => plugin.constructor.name === 'ModuleScopePlugin'
      );
      
      if (moduleScopePluginIndex !== -1) {
        webpackConfig.resolve.plugins.splice(moduleScopePluginIndex, 1);
      }
      
      // Add public directory to module resolution
      if (!webpackConfig.resolve.modules) {
        webpackConfig.resolve.modules = [];
      }
      webpackConfig.resolve.modules.push(path.resolve(__dirname, 'public'));
      
      // Add alias for easier imports
      if (!webpackConfig.resolve.alias) {
        webpackConfig.resolve.alias = {};
      }
      webpackConfig.resolve.alias['@public'] = path.resolve(__dirname, 'public');
      
      // Override the resolve configuration more aggressively
      webpackConfig.resolve = {
        ...webpackConfig.resolve,
        modules: [
          'node_modules',
          path.resolve(__dirname, 'public'),
          ...(webpackConfig.resolve.modules || [])
        ],
        alias: {
          ...webpackConfig.resolve.alias,
          '@public': path.resolve(__dirname, 'public')
        }
      };

      // Increase TypeScript checker memory to prevent OOM in large projects
      try {
        const ftcIndex = (webpackConfig.plugins || []).findIndex(
          (p) => p && p.constructor && p.constructor.name === 'ForkTsCheckerWebpackPlugin'
        );
        const newOptions = (old) => ({
          ...(old && old.options ? old.options : {}),
          typescript: {
            ...((old && old.options && old.options.typescript) || {}),
            memoryLimit: 4096,
          },
        });
        if (ftcIndex !== -1) {
          const old = webpackConfig.plugins[ftcIndex];
          webpackConfig.plugins[ftcIndex] = new ForkTsCheckerWebpackPlugin(newOptions(old));
        } else {
          (webpackConfig.plugins = webpackConfig.plugins || []).push(
            new ForkTsCheckerWebpackPlugin(newOptions(null))
          );
        }
      } catch (e) {
        // Non-fatal if plugin structure changes; proceed with defaults
        console.warn('[craco] Failed to adjust ForkTsCheckerWebpackPlugin memoryLimit:', e?.message || e);
      }
      
      return webpackConfig;
    },
  },
}; 