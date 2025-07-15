// Simple test to verify environment configuration
import { env } from './config/environment.js';

console.log('Environment Configuration Test:');
console.log('Environment:', env.environment);
console.log('Is Windows:', env.isWindows);
console.log('Is Web:', env.isWeb);
console.log('Is Production:', env.isProduction);
console.log('Config:', JSON.stringify(env.config, null, 2)); 