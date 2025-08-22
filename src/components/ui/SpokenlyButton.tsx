/**
 * SpokenlyButton - Primary button component
 * Provides various button variants with loading states and icons
 */

import React from 'react';
import { motion } from 'framer-motion';
import { ButtonProps } from './types';

export const SpokenlyButton: React.FC<ButtonProps> = ({
  children,
  className = '',
  variant = 'primary',
  size = 'md',
  isLoading = false,
  loadingText = 'Loading...',
  leftIcon,
  rightIcon,
  fullWidth = false,
  disabled,
  onClick,
  ...props
}) => {
  const isDisabled = disabled || isLoading;

  // Size mappings
  const sizeStyles = {
    xs: {
      padding: '6px 12px',
      fontSize: 'var(--spokenly-text-xs)',
      minHeight: '28px'
    },
    sm: {
      padding: '8px 16px',
      fontSize: 'var(--spokenly-text-sm)',
      minHeight: '32px'
    },
    md: {
      padding: '10px 20px',
      fontSize: 'var(--spokenly-text-base)',
      minHeight: '40px'
    },
    lg: {
      padding: '12px 24px',
      fontSize: 'var(--spokenly-text-lg)',
      minHeight: '48px'
    }
  };

  // Variant styles
  const getVariantStyles = () => {
    const baseStyles = {
      border: 'none',
      borderRadius: 'var(--spokenly-radius-base)',
      fontFamily: 'var(--spokenly-font-family)',
      fontWeight: 'var(--spokenly-font-medium)',
      cursor: isDisabled ? 'not-allowed' : 'pointer',
      transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
      display: 'inline-flex',
      alignItems: 'center',
      justifyContent: 'center',
      textDecoration: 'none',
      position: 'relative' as const,
      overflow: 'hidden',
      ...sizeStyles[size]
    };

    switch (variant) {
      case 'primary':
        return {
          ...baseStyles,
          backgroundColor: isDisabled ? 'var(--spokenly-border-secondary)' : 'var(--spokenly-primary)',
          color: 'var(--spokenly-text-white)',
          boxShadow: isDisabled ? 'none' : 'var(--spokenly-shadow-sm)'
        };
      
      case 'secondary':
        return {
          ...baseStyles,
          backgroundColor: isDisabled ? 'var(--spokenly-bg-hover)' : 'var(--spokenly-bg-content)',
          color: isDisabled ? 'var(--spokenly-text-tertiary)' : 'var(--spokenly-text-primary)',
          border: `1px solid ${isDisabled ? 'var(--spokenly-border-secondary)' : 'var(--spokenly-border-primary)'}`
        };
      
      case 'ghost':
        return {
          ...baseStyles,
          backgroundColor: 'transparent',
          color: isDisabled ? 'var(--spokenly-text-tertiary)' : 'var(--spokenly-text-secondary)'
        };
      
      case 'danger':
        return {
          ...baseStyles,
          backgroundColor: isDisabled ? 'var(--spokenly-border-secondary)' : 'var(--spokenly-error)',
          color: 'var(--spokenly-text-white)'
        };
      
      case 'success':
        return {
          ...baseStyles,
          backgroundColor: isDisabled ? 'var(--spokenly-border-secondary)' : 'var(--spokenly-success)',
          color: 'var(--spokenly-text-white)'
        };
      
      default:
        return baseStyles;
    }
  };

  // Hover styles
  const getHoverStyles = () => {
    if (isDisabled) return {};
    
    switch (variant) {
      case 'primary':
        return { backgroundColor: 'var(--spokenly-primary-hover)' };
      case 'secondary':
        return { 
          backgroundColor: 'var(--spokenly-bg-hover)',
          borderColor: 'var(--spokenly-border-secondary)'
        };
      case 'ghost':
        return { 
          backgroundColor: 'var(--spokenly-bg-hover)',
          color: 'var(--spokenly-text-primary)'
        };
      case 'danger':
        return { backgroundColor: '#E60026' };
      case 'success':
        return { backgroundColor: '#2DA642' };
      default:
        return {};
    }
  };

  const buttonContent = (
    <>
      {/* Left Icon or Loading Spinner */}
      {(leftIcon || isLoading) && (
        <span 
          className="spokenly-button-icon-left"
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            marginRight: children ? 'var(--spokenly-space-2)' : '0'
          }}
        >
          {isLoading ? (
            <motion.div
              animate={{ rotate: 360 }}
              transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
              style={{
                width: size === 'xs' ? '12px' : size === 'sm' ? '14px' : '16px',
                height: size === 'xs' ? '12px' : size === 'sm' ? '14px' : '16px',
                border: '2px solid currentColor',
                borderTopColor: 'transparent',
                borderRadius: '50%'
              }}
            />
          ) : leftIcon}
        </span>
      )}

      {/* Button Text */}
      <span className="spokenly-button-text">
        {isLoading ? loadingText : children}
      </span>

      {/* Right Icon */}
      {rightIcon && !isLoading && (
        <span 
          className="spokenly-button-icon-right"
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            marginLeft: children ? 'var(--spokenly-space-2)' : '0'
          }}
        >
          {rightIcon}
        </span>
      )}
    </>
  );

  return (
    <motion.button
      className={`spokenly-button spokenly-button--${variant} spokenly-button--${size} ${className}`}
      style={{
        ...getVariantStyles(),
        width: fullWidth ? '100%' : 'auto',
        opacity: isDisabled ? 0.6 : 1
      }}
      whileHover={getHoverStyles()}
      whileTap={isDisabled ? {} : { scale: 0.98 }}
      whileFocus={{ 
        boxShadow: 'var(--spokenly-shadow-focus)'
      }}
      disabled={isDisabled}
      onClick={onClick}
      {...props}
    >
      {buttonContent}
    </motion.button>
  );
};