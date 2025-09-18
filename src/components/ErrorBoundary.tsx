import React, { Component, ErrorInfo, ReactNode } from "react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
  componentName?: string;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    console.error("ErrorBoundary caught an error:", error);
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    console.error("ErrorBoundary详细错误信息:", {
      error,
      errorInfo,
      componentName: this.props.componentName,
      timestamp: new Date().toISOString(),
    });

    this.setState({
      error,
      errorInfo,
    });
  }

  render() {
    if (this.state.hasError) {
      const fallback = this.props.fallback || (
        <div
          style={{
            padding: "20px",
            border: "2px solid #ff6b6b",
            borderRadius: "8px",
            backgroundColor: "#ffe0e0",
            margin: "10px 0",
          }}
        >
          <h3 style={{ color: "#d63031", marginTop: 0 }}>
            🚨 {this.props.componentName || "组件"} 加载失败
          </h3>
          <details style={{ marginTop: "10px" }}>
            <summary style={{ cursor: "pointer", fontWeight: "bold" }}>
              点击查看错误详情
            </summary>
            <pre
              style={{
                background: "#f8f8f8",
                padding: "10px",
                borderRadius: "4px",
                overflow: "auto",
                fontSize: "12px",
                marginTop: "10px",
              }}
            >
              {this.state.error?.message}
              {this.state.errorInfo?.componentStack}
            </pre>
          </details>
          <button
            onClick={() =>
              this.setState({
                hasError: false,
                error: undefined,
                errorInfo: undefined,
              })
            }
            style={{
              marginTop: "10px",
              padding: "8px 16px",
              backgroundColor: "#74b9ff",
              color: "white",
              border: "none",
              borderRadius: "4px",
              cursor: "pointer",
            }}
          >
            🔄 重试加载
          </button>
        </div>
      );

      return fallback;
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
