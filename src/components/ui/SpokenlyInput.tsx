/**
 * SpokenlyInput - Input field component
 * Provides text input with icons, labels, and validation states
 */

import React, { useState, useRef } from 'react';
import { motion } from 'framer-motion';
import { InputProps } from './types';

export const SpokenlyInput: React.FC<InputProps> = ({
  className = '',
  label,
  helperText,
  errorText,
  leftIcon,
  rightIcon,
  size = 'md',
  fullWidth = false,
  disabled = false,
  onFocus,
  onBlur,
  ...props
}) => {
  const [isFocused, setIsFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const hasError = !!errorText;

  // Size mappings
  const sizeStyles = {
    sm: {
      fontSize: 'var(--spokenly-text-sm)',
      padding: '8px 12px',
      minHeight: '32px'
    },
    md: {
      fontSize: 'var(--spokenly-text-base)',
      padding: '10px 16px',
      minHeight: '40px'
    },
    lg: {
      fontSize: 'var(--spokenly-text-lg)',
      padding: '12px 20px',
      minHeight: '48px'
    }
  };

  const inputStyles = {
    width: fullWidth ? '100%' : 'auto',
    fontFamily: 'var(--spokenly-font-family)',
    fontWeight: 'var(--spokenly-font-normal)',
    color: disabled ? 'var(--spokenly-text-tertiary)' : 'var(--spokenly-text-primary)',
    backgroundColor: disabled ? 'var(--spokenly-bg-hover)' : 'var(--spokenly-bg-content)',
    border: `1px solid ${hasError ? 'var(--spokenly-error)' : isFocused ? 'var(--spokenly-primary)' : 'var(--spokenly-border-primary)'}`,
    borderRadius: 'var(--spokenly-radius-base)',
    outline: 'none',
    transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
    position: 'relative' as const,
    ...sizeStyles[size]
  };

  // Calculate padding for icons
  const iconSize = size === 'sm' ? '16px' : size === 'lg' ? '20px' : '18px';
  const iconPadding = size === 'sm' ? '36px' : size === 'lg' ? '52px' : '44px';
  
  if (leftIcon) {
    inputStyles.paddingLeft = iconPadding;
  }
  if (rightIcon) {
    inputStyles.paddingRight = iconPadding;
  }

  const handleFocus = (e: React.FocusEvent<HTMLInputElement>) => {
    setIsFocused(true);
    onFocus?.(e);
  };

  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    setIsFocused(false);
    onBlur?.(e);
  };

  return (
    <div 
      className={`spokenly-input-container ${className}`}
      style={{
        width: fullWidth ? '100%' : 'auto',
        position: 'relative'
      }}
    >
      {/* Label */}
      {label && (
        <motion.label
          className="spokenly-input-label"
          htmlFor={props.id}
          initial={false}
          animate={{
            color: hasError ? 'var(--spokenly-error)' : isFocused ? 'var(--spokenly-primary)' : 'var(--spokenly-text-secondary)'
          }}
          style={{
            display: 'block',
            fontSize: 'var(--spokenly-text-sm)',
            fontWeight: 'var(--spokenly-font-medium)',
            marginBottom: 'var(--spokenly-space-2)',
            transition: 'color var(--spokenly-duration-fast) var(--spokenly-ease-out)'
          }}
        >
          {label}
        </motion.label>
      )}

      {/* Input Container */}
      <div 
        className="spokenly-input-wrapper"
        style={{
          position: 'relative',
          width: fullWidth ? '100%' : 'auto'
        }}
      >
        {/* Left Icon */}
        {leftIcon && (
          <div
            className="spokenly-input-icon-left"
            style={{
              position: 'absolute',
              left: '12px',
              top: '50%',
              transform: 'translateY(-50%)',
              fontSize: iconSize,
              color: hasError ? 'var(--spokenly-error)' : isFocused ? 'var(--spokenly-primary)' : 'var(--spokenly-text-tertiary)',
              transition: 'color var(--spokenly-duration-fast) var(--spokenly-ease-out)',
              pointerEvents: 'none',
              display: 'flex',
              alignItems: 'center',
              zIndex: 1
            }}
          >
            {leftIcon}
          </div>
        )}

        {/* Input Field */}
        <motion.input
          ref={inputRef}
          className="spokenly-input"
          style={inputStyles}
          disabled={disabled}
          onFocus={handleFocus}
          onBlur={handleBlur}
          whileFocus={{
            boxShadow: hasError ? '0 0 0 3px rgba(255, 59, 48, 0.15)' : 'var(--spokenly-shadow-focus)'
          }}
          {...props}
        />

        {/* Right Icon */}
        {rightIcon && (
          <div
            className="spokenly-input-icon-right"
            style={{
              position: 'absolute',
              right: '12px',
              top: '50%',
              transform: 'translateY(-50%)',
              fontSize: iconSize,
              color: hasError ? 'var(--spokenly-error)' : isFocused ? 'var(--spokenly-primary)' : 'var(--spokenly-text-tertiary)',
              transition: 'color var(--spokenly-duration-fast) var(--spokenly-ease-out)',
              pointerEvents: 'none',
              display: 'flex',
              alignItems: 'center',
              zIndex: 1
            }}
          >
            {rightIcon}
          </div>
        )}
      </div>

      {/* Helper/Error Text */}
      {(helperText || errorText) && (
        <motion.p
          className={`spokenly-input-help ${hasError ? 'error' : ''}`}
          initial={{ opacity: 0, y: -5 }}
          animate={{ opacity: 1, y: 0 }}
          style={{
            fontSize: 'var(--spokenly-text-sm)',
            color: hasError ? 'var(--spokenly-error)' : 'var(--spokenly-text-secondary)',
            margin: '4px 0 0 0',
            lineHeight: 'var(--spokenly-leading-normal)'
          }}
        >
          {errorText || helperText}
        </motion.p>
      )}
    </div>
  );
};