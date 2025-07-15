      onError={(error: Error, errorInfo: React.ErrorInfo) => {
        logger.error(`Error in ${isWindows ? 'Windows' : 'Web'} environment:`, { errorInfo, error: error.message }, error);
        onError?.(error, errorInfo);
      }}