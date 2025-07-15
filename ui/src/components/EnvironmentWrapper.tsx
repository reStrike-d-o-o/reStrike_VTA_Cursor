import React from 'react';
import { useEnvironment } from '../hooks/useEnvironment';

interface EnvironmentWrapperProps {
  children: React.ReactNode;
  windowsOnly?: boolean;
  webOnly?: boolean;
  fallback?: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

// Environment-aware component wrapper
export const EnvironmentWrapper: React.FC<EnvironmentWrapperProps> = ({
  children,
  windowsOnly = false,
  webOnly = false,
  fallback = null,
  className = '',
  style = {},
}) => {
  const { isWindows, isWeb, isClient, getEnvironmentClass } = useEnvironment();

  // Don't render anything on server-side
  if (!isClient) {
    return null;
  }

  // Check if component should be rendered in current environment
  const shouldRender = 
    (!windowsOnly && !webOnly) || // Render in all environments
    (windowsOnly && isWindows) || // Windows-only component
    (webOnly && isWeb); // Web-only component

  if (!shouldRender) {
    return fallback ? <>{fallback}</> : null;
  }

  return (
    <div 
      className={getEnvironmentClass(className)}
      style={style}
      data-environment={isWindows ? 'windows' : 'web'}
    >
      {children}
    </div>
  );
};

// Windows-only component wrapper
export const WindowsOnly: React.FC<Omit<EnvironmentWrapperProps, 'windowsOnly'>> = (props) => (
  <EnvironmentWrapper {...props} windowsOnly={true} />
);

// Web-only component wrapper
export const WebOnly: React.FC<Omit<EnvironmentWrapperProps, 'webOnly'>> = (props) => (
  <EnvironmentWrapper {...props} webOnly={true} />
);

// Environment-specific feature wrapper
interface FeatureWrapperProps {
  children: React.ReactNode;
  feature: string;
  fallback?: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}

export const FeatureWrapper: React.FC<FeatureWrapperProps> = ({
  children,
  feature,
  fallback = null,
  className = '',
  style = {},
}) => {
  const { config } = useEnvironment();
  
  // Check if feature is available in current environment
  const isFeatureAvailable = config.features[feature as keyof typeof config.features];
  
  if (!isFeatureAvailable) {
    return fallback ? <>{fallback}</> : null;
  }

  return (
    <div 
      className={`feature-wrapper feature-wrapper--${feature} ${className}`}
      style={style}
      data-feature={feature}
    >
      {children}
    </div>
  );
};

// Environment-aware conditional rendering
export const Conditional: React.FC<{
  condition: boolean;
  children: React.ReactNode;
  fallback?: React.ReactNode;
}> = ({ condition, children, fallback = null }) => {
  return condition ? <>{children}</> : <>{fallback}</>;
};

// Environment-aware loading wrapper
export const LoadingWrapper: React.FC<{
  children: React.ReactNode;
  loading: boolean;
  fallback?: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
}> = ({ children, loading, fallback = <div>Loading...</div>, className = '', style = {} }) => {
  const { getEnvironmentClass } = useEnvironment();
  
  return (
    <div 
      className={getEnvironmentClass(`loading-wrapper ${className}`)}
      style={style}
    >
      {loading ? fallback : children}
    </div>
  );
};

// Environment-aware error boundary
class ErrorBoundaryClass extends React.Component<
  { 
    children: React.ReactNode; 
    fallback: React.ReactNode;
    onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
  },
  { hasError: boolean }
> {
  constructor(props: any) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo);
    this.props.onError?.(error, errorInfo);
  }

  render() {
    if (this.state.hasError) {
      return this.props.fallback;
    }

    return this.props.children;
  }
}

export const ErrorBoundary: React.FC<{
  children: React.ReactNode;
  fallback?: React.ReactNode;
  onError?: (error: Error, errorInfo: React.ErrorInfo) => void;
}> = ({ children, fallback = <div>Something went wrong.</div>, onError }) => {
  const { isWindows, isWeb } = useEnvironment();
  
  return (
    <ErrorBoundaryClass
      fallback={fallback}
      onError={(error: Error, errorInfo: React.ErrorInfo) => {
        console.error(`Error in ${isWindows ? 'Windows' : 'Web'} environment:`, error, errorInfo);
        onError?.(error, errorInfo);
      }}
    >
      {children}
    </ErrorBoundaryClass>
  );
}; 