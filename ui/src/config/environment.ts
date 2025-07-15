// Environment-specific logging
export const log = (message: string, data?: any) => {
  logger.info(message, data, 'Environment');
};

export const logError = (message: string, error?: any) => {
  logger.error(message, { error }, 'Environment', error instanceof Error ? error : undefined);
};