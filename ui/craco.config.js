import path from 'path';
import { fileURLToPath } from 'url';

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
      
      return webpackConfig;
    },
  },
}; 