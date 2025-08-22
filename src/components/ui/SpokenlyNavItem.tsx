/**
 * SpokenlyNavItem - Navigation menu item
 * Provides interactive navigation items with icons, labels, and states
 */

import React from 'react';
import { motion } from 'framer-motion';
import { NavItemProps } from './types';

export const SpokenlyNavItem: React.FC<NavItemProps> = ({
  icon,
  label,
  isActive = false,
  isDisabled = false,
  onClick,
  href,
  badge,
  className = ''
}) => {
  const Component = href ? motion.a : motion.button;
  
  const baseStyles = {
    display: 'flex',
    alignItems: 'center',
    width: '100%',
    padding: 'var(--spokenly-space-3) var(--spokenly-space-4)',
    borderRadius: 'var(--spokenly-radius-base)',
    border: 'none',
    textDecoration: 'none',
    fontSize: 'var(--spokenly-text-sm)',
    fontWeight: 'var(--spokenly-font-medium)',
    cursor: isDisabled ? 'not-allowed' : 'pointer',
    transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
    position: 'relative' as const,
    marginBottom: 'var(--spokenly-space-1)'
  };

  const getItemStyles = () => {
    if (isDisabled) {
      return {
        ...baseStyles,
        backgroundColor: 'transparent',
        color: 'var(--spokenly-text-tertiary)',
        opacity: 0.5
      };
    }

    if (isActive) {
      return {
        ...baseStyles,
        backgroundColor: 'var(--spokenly-bg-selected)',
        color: 'var(--spokenly-primary)',
        fontWeight: 'var(--spokenly-font-semibold)'
      };
    }

    return {
      ...baseStyles,
      backgroundColor: 'transparent',
      color: 'var(--spokenly-text-secondary)'
    };
  };

  const hoverStyles = !isDisabled ? {
    backgroundColor: isActive ? 'var(--spokenly-bg-selected)' : 'var(--spokenly-bg-hover)',
    color: isActive ? 'var(--spokenly-primary)' : 'var(--spokenly-text-primary)'
  } : {};

  const tapStyles = !isDisabled ? { scale: 0.98 } : {};

  return (
    <Component
      className={`spokenly-nav-item ${className} ${isActive ? 'active' : ''} ${isDisabled ? 'disabled' : ''}`}
      style={getItemStyles()}
      whileHover={hoverStyles}
      whileTap={tapStyles}
      onClick={!isDisabled ? onClick : undefined}
      href={href}
      disabled={isDisabled}
      initial={false}
    >
      {/* Icon */}
      {icon && (
        <span 
          className="spokenly-nav-item-icon"
          style={{
            display: 'flex',
            alignItems: 'center',
            marginRight: 'var(--spokenly-space-3)',
            fontSize: '16px',
            opacity: isActive ? 1 : 0.7,
            transition: 'opacity var(--spokenly-duration-fast) var(--spokenly-ease-out)'
          }}
        >
          {icon}
        </span>
      )}

      {/* Label */}
      <span 
        className="spokenly-nav-item-label"
        style={{
          flex: 1,
          textAlign: 'left' as const,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap' as const
        }}
      >
        {label}
      </span>

      {/* Badge */}
      {badge && (
        <motion.span
          className="spokenly-nav-item-badge"
          initial={{ scale: 0 }}
          animate={{ scale: 1 }}
          style={{
            backgroundColor: 'var(--spokenly-primary)',
            color: 'var(--spokenly-text-white)',
            fontSize: 'var(--spokenly-text-xs)',
            fontWeight: 'var(--spokenly-font-semibold)',
            padding: '2px 6px',
            borderRadius: 'var(--spokenly-radius-full)',
            minWidth: '18px',
            height: '18px',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            marginLeft: 'var(--spokenly-space-2)'
          }}
        >
          {badge}
        </motion.span>
      )}

      {/* Active Indicator */}
      {isActive && (
        <motion.div
          className="spokenly-nav-item-indicator"
          layoutId="activeIndicator"
          style={{
            position: 'absolute',
            left: 0,
            top: 0,
            bottom: 0,
            width: '3px',
            backgroundColor: 'var(--spokenly-primary)',
            borderRadius: '0 2px 2px 0'
          }}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.2 }}
        />
      )}
    </Component>
  );
};