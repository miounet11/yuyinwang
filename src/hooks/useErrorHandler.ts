import { useCallback } from 'react';
import { message } from '@tauri-apps/api/dialog';

export interface ErrorInfo {
  message: string;
  code?: string;
  details?: any;
}

export const useErrorHandler = () => {
  const handleError = useCallback(async (error: unknown, context?: string) => {
    let errorMessage = '发生了未知错误';
    let errorDetails = '';
    
    if (error instanceof Error) {
      errorMessage = error.message;
      errorDetails = error.stack || '';
    } else if (typeof error === 'string') {
      errorMessage = error;
    } else if (error && typeof error === 'object') {
      errorMessage = (error as any).message || JSON.stringify(error);
      errorDetails = (error as any).stack || '';
    }
    
    // 记录错误到控制台
    console.error(`[${context || 'Unknown'}] Error:`, error);
    
    // 开发环境显示详细错误
    if (process.env.NODE_ENV === 'development') {
      console.error('Error Details:', errorDetails);
    }
    
    // 显示用户友好的错误消息
    await message(
      `${context ? `[${context}] ` : ''}${errorMessage}`,
      { title: '错误', type: 'error' }
    );
    
    // 返回错误信息，便于进一步处理
    return {
      message: errorMessage,
      details: errorDetails,
      context
    };
  }, []);
  
  const handleAsyncOperation = useCallback(async <T,>(
    operation: () => Promise<T>,
    context?: string
  ): Promise<T | null> => {
    try {
      return await operation();
    } catch (error) {
      await handleError(error, context);
      return null;
    }
  }, [handleError]);
  
  return {
    handleError,
    handleAsyncOperation
  };
};