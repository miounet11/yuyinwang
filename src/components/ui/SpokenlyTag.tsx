/**
 * SpokenlyTag - Tag/Badge component
 * Provides colored tags with removable functionality and icons
 */

import React from 'react';
import { motion } from 'framer-motion';
import { TagProps } from './types';

export const SpokenlyTag: React.FC<TagProps> = ({
  children,
  className = '',
  variant = 'default',
  size = 'md',
  removable = false,
  onRemove,
  icon
}) => {
  // Size mappings
  const sizeStyles = {
    sm: {
      fontSize: 'var(--spokenly-text-xs)',
      padding: '2px 6px',
      height: '20px',
      iconSize: '12px'
    },
    md: {
      fontSize: 'var(--spokenly-text-sm)',
      padding: '4px 8px',
      height: '24px',
      iconSize: '14px'
    },
    lg: {
      fontSize: 'var(--spokenly-text-base)',
      padding: '6px 12px',
      height: '32px',
      iconSize: '16px'
    }
  };

  // Variant styles
  const getVariantStyles = () => {
    const baseStyles = {
      borderRadius: 'var(--spokenly-radius-full)',
      fontFamily: 'var(--spokenly-font-family)',
      fontWeight: 'var(--spokenly-font-medium)',
      display: 'inline-flex',
      alignItems: 'center',
      gap: 'var(--spokenly-space-1)',
      border: '1px solid transparent',
      transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
      ...sizeStyles[size]
    };

    switch (variant) {
      case 'default':
        return {
          ...baseStyles,
          backgroundColor: 'var(--spokenly-bg-hover)',
          color: 'var(--spokenly-text-secondary)',
          borderColor: 'var(--spokenly-border-primary)'
        };
      
      case 'success':
        return {
          ...baseStyles,
          backgroundColor: 'rgba(52, 199, 89, 0.1)',
          color: 'var(--spokenly-success)',
          borderColor: 'rgba(52, 199, 89, 0.2)'
        };
      
      case 'warning':
        return {
          ...baseStyles,
          backgroundColor: 'rgba(255, 149, 0, 0.1)',
          color: 'var(--spokenly-warning)',
          borderColor: 'rgba(255, 149, 0, 0.2)'
        };
      
      case 'error':
        return {
          ...baseStyles,
          backgroundColor: 'rgba(255, 59, 48, 0.1)',
          color: 'var(--spokenly-error)',
          borderColor: 'rgba(255, 59, 48, 0.2)'
        };
      
      case 'info':
        return {
          ...baseStyles,
          backgroundColor: 'rgba(0, 122, 255, 0.1)',
          color: 'var(--spokenly-primary)',
          borderColor: 'rgba(0, 122, 255, 0.2)'
        };
      
      default:
        return baseStyles;
    }
  };

  const handleRemove = (e: React.MouseEvent) => {
    e.stopPropagation();
    onRemove?.();
  };

  return (
    <motion.span
      className={`spokenly-tag spokenly-tag--${variant} spokenly-tag--${size} ${className}`}
      style={getVariantStyles()}
      initial={{ scale: 0, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      exit={{ scale: 0, opacity: 0 }}
      whileHover={{ scale: 1.05 }}
      whileTap={{ scale: 0.95 }}
      layout
    >
      {/* Icon */}
      {icon && (
        <span 
          className="spokenly-tag-icon"
          style={{
            display: 'flex',
            alignItems: 'center',
            fontSize: sizeStyles[size].iconSize
          }}
        >
          {icon}
        </span>
      )}

      {/* Content */}
      <span className="spokenly-tag-content">
        {children}
      </span>

      {/* Remove Button */}
      {removable && (
        <motion.button
          className="spokenly-tag-remove"
          onClick={handleRemove}
          style={{
            background: 'none',
            border: 'none',
            padding: 0,
            margin: 0,
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            width: sizeStyles[size].iconSize,
            height: sizeStyles[size].iconSize,
            borderRadius: '50%',
            color: 'currentColor',
            opacity: 0.7,
            transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)'
          }}
          whileHover={{ 
            opacity: 1,
            backgroundColor: 'rgba(0, 0, 0, 0.1)',
            scale: 1.1 
          }}
          whileTap={{ scale: 0.9 }}
        >
          <svg 
            width={sizeStyles[size].iconSize} 
            height={sizeStyles[size].iconSize} 
            viewBox="0 0 16 16" 
            fill="currentColor"
          >
            <path d="M12 4L4 12m8 0L4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" fill="none"/>
          </svg>
        </motion.button>
      )}
    </motion.span>
  );
};