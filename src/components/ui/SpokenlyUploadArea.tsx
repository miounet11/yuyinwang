/**
 * SpokenlyUploadArea - File upload area with drag & drop
 * Provides file upload functionality with visual feedback
 */

import React, { useState, useRef, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { UploadAreaProps } from './types';

export const SpokenlyUploadArea: React.FC<UploadAreaProps> = ({
  onFilesDrop,
  onFilesSelect,
  accept,
  multiple = false,
  maxSize,
  disabled = false,
  title = 'Drop files here',
  description = 'or click to select files',
  icon,
  className = ''
}) => {
  const [isDragOver, setIsDragOver] = useState(false);
  const [isError, setIsError] = useState(false);
  const [errorMessage, setErrorMessage] = useState('');
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Format file size
  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  // Validate files
  const validateFiles = (files: FileList) => {
    if (maxSize) {
      for (let i = 0; i < files.length; i++) {
        if (files[i].size > maxSize) {
          setIsError(true);
          setErrorMessage(`File "${files[i].name}" is too large. Maximum size is ${formatFileSize(maxSize)}.`);
          return false;
        }
      }
    }
    
    setIsError(false);
    setErrorMessage('');
    return true;
  };

  // Handle drag events
  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (!disabled) {
      setIsDragOver(true);
    }
  }, [disabled]);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (!disabled) {
      setIsDragOver(false);
    }
  }, [disabled]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    
    if (disabled) return;
    
    setIsDragOver(false);
    const files = e.dataTransfer.files;
    
    if (files && validateFiles(files)) {
      onFilesDrop?.(files);
    }
  }, [disabled, onFilesDrop, maxSize]);

  // Handle file selection
  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && validateFiles(files)) {
      onFilesSelect?.(files);
    }
  }, [onFilesSelect, maxSize]);

  // Handle click to select files
  const handleClick = () => {
    if (!disabled) {
      fileInputRef.current?.click();
    }
  };

  // Default icon
  const defaultIcon = (
    <svg width="48" height="48" viewBox="0 0 48 48" fill="currentColor">
      <path d="M24 4L20 8H8C5.8 8 4 9.8 4 12V36C4 38.2 5.8 40 8 40H40C42.2 40 44 38.2 44 36V16C44 13.8 42.2 12 40 12H28L24 8Z" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
      <path d="M20 24L24 20L28 24" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
      <path d="M24 20V32" stroke="currentColor" strokeWidth="2" strokeLinecap="round" fill="none"/>
    </svg>
  );

  return (
    <div 
      className={`spokenly-upload-area ${className}`}
      style={{ width: '100%' }}
    >
      <motion.div
        className={`spokenly-upload-container ${isDragOver ? 'drag-over' : ''} ${disabled ? 'disabled' : ''} ${isError ? 'error' : ''}`}
        style={{
          border: `2px dashed ${isError ? 'var(--spokenly-error)' : isDragOver ? 'var(--spokenly-primary)' : 'var(--spokenly-border-secondary)'}`,
          borderRadius: 'var(--spokenly-radius-lg)',
          padding: 'var(--spokenly-space-12)',
          textAlign: 'center' as const,
          cursor: disabled ? 'not-allowed' : 'pointer',
          transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
          backgroundColor: isDragOver 
            ? 'rgba(0, 122, 255, 0.05)' 
            : disabled 
            ? 'var(--spokenly-bg-hover)'
            : 'transparent',
          opacity: disabled ? 0.6 : 1
        }}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onClick={handleClick}
        whileHover={disabled ? {} : {
          borderColor: 'var(--spokenly-primary)',
          backgroundColor: 'rgba(0, 122, 255, 0.02)'
        }}
        whileTap={disabled ? {} : { scale: 0.995 }}
      >
        {/* Icon */}
        <motion.div
          className="spokenly-upload-icon"
          style={{
            color: isError ? 'var(--spokenly-error)' : isDragOver ? 'var(--spokenly-primary)' : 'var(--spokenly-text-tertiary)',
            marginBottom: 'var(--spokenly-space-4)',
            transition: 'color var(--spokenly-duration-fast) var(--spokenly-ease-out)'
          }}
          animate={{
            scale: isDragOver ? 1.1 : 1,
            rotate: isDragOver ? [0, -5, 5, 0] : 0
          }}
          transition={{ duration: 0.3 }}
        >
          {icon || defaultIcon}
        </motion.div>

        {/* Title */}
        <h3
          className="spokenly-upload-title"
          style={{
            fontSize: 'var(--spokenly-text-lg)',
            fontWeight: 'var(--spokenly-font-semibold)',
            color: isError ? 'var(--spokenly-error)' : isDragOver ? 'var(--spokenly-primary)' : 'var(--spokenly-text-primary)',
            margin: '0 0 var(--spokenly-space-2) 0',
            transition: 'color var(--spokenly-duration-fast) var(--spokenly-ease-out)'
          }}
        >
          {title}
        </h3>

        {/* Description */}
        <p
          className="spokenly-upload-description"
          style={{
            fontSize: 'var(--spokenly-text-base)',
            color: 'var(--spokenly-text-secondary)',
            margin: '0 0 var(--spokenly-space-4) 0',
            lineHeight: 'var(--spokenly-leading-normal)'
          }}
        >
          {description}
        </p>

        {/* File Info */}
        <div
          className="spokenly-upload-info"
          style={{
            fontSize: 'var(--spokenly-text-sm)',
            color: 'var(--spokenly-text-tertiary)',
            margin: 0
          }}
        >
          {accept && (
            <div>
              Accepted formats: {accept.split(',').join(', ')}
            </div>
          )}
          {maxSize && (
            <div style={{ marginTop: 'var(--spokenly-space-1)' }}>
              Maximum file size: {formatFileSize(maxSize)}
            </div>
          )}
          {multiple && (
            <div style={{ marginTop: 'var(--spokenly-space-1)' }}>
              Multiple files allowed
            </div>
          )}
        </div>

        {/* Loading Overlay */}
        <AnimatePresence>
          {isDragOver && (
            <motion.div
              className="spokenly-upload-overlay"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              style={{
                position: 'absolute',
                inset: 0,
                backgroundColor: 'rgba(0, 122, 255, 0.1)',
                borderRadius: 'inherit',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                pointerEvents: 'none'
              }}
            >
              <motion.div
                style={{
                  width: '60px',
                  height: '60px',
                  border: '3px solid var(--spokenly-primary)',
                  borderRadius: '50%',
                  borderTopColor: 'transparent',
                }}
                animate={{ rotate: 360 }}
                transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
              />
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>

      {/* Error Message */}
      <AnimatePresence>
        {isError && errorMessage && (
          <motion.div
            className="spokenly-upload-error"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            style={{
              marginTop: 'var(--spokenly-space-3)',
              padding: 'var(--spokenly-space-3)',
              backgroundColor: 'rgba(255, 59, 48, 0.1)',
              border: '1px solid rgba(255, 59, 48, 0.2)',
              borderRadius: 'var(--spokenly-radius-base)',
              color: 'var(--spokenly-error)',
              fontSize: 'var(--spokenly-text-sm)',
              textAlign: 'center' as const
            }}
          >
            {errorMessage}
          </motion.div>
        )}
      </AnimatePresence>

      {/* Hidden File Input */}
      <input
        ref={fileInputRef}
        type="file"
        accept={accept}
        multiple={multiple}
        onChange={handleFileSelect}
        style={{ display: 'none' }}
        disabled={disabled}
      />
    </div>
  );
};