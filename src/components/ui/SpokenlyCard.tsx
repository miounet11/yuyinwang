/**
 * SpokenlyCard - Card container component
 * Provides flexible card layout with header, body, and footer sections
 */

import React from 'react';
import { motion } from 'framer-motion';
import { CardProps, CardHeaderProps, CardBodyProps, CardFooterProps } from './types';

// Main Card Component
export const SpokenlyCard: React.FC<CardProps> = ({
  children,
  className = '',
  padding = 'md',
  hover = false,
  selected = false,
  onClick,
  ...props
}) => {
  const getPadding = (size: string) => {
    switch (size) {
      case 'sm': return 'var(--spokenly-space-4)';
      case 'lg': return 'var(--spokenly-space-8)';
      default: return 'var(--spokenly-card-padding)';
    }
  };

  const cardStyles = {
    backgroundColor: 'var(--spokenly-bg-card)',
    borderRadius: 'var(--spokenly-radius-lg)',
    border: `1px solid ${selected ? 'var(--spokenly-border-selected)' : 'var(--spokenly-border-primary)'}`,
    boxShadow: selected ? 'var(--spokenly-shadow-lg)' : 'var(--spokenly-shadow-sm)',
    padding: getPadding(padding),
    cursor: onClick ? 'pointer' : 'default',
    transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
    position: 'relative' as const,
    overflow: 'hidden'
  };

  const hoverStyles = hover && !selected ? {
    boxShadow: 'var(--spokenly-shadow-hover)',
    transform: 'translateY(-2px)',
    borderColor: 'var(--spokenly-border-secondary)'
  } : {};

  return (
    <motion.div
      className={`spokenly-card ${className} ${selected ? 'selected' : ''} ${hover ? 'hoverable' : ''}`}
      style={cardStyles}
      whileHover={hoverStyles}
      whileTap={onClick ? { scale: 0.995 } : {}}
      onClick={onClick}
      layout
      {...props}
    >
      {children}
    </motion.div>
  );
};

// Card Header Component
export const SpokenlyCardHeader: React.FC<CardHeaderProps> = ({
  title,
  subtitle,
  actions,
  children,
  className = ''
}) => {
  return (
    <div 
      className={`spokenly-card-header ${className}`}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        marginBottom: 'var(--spokenly-space-4)',
        paddingBottom: title || subtitle ? 'var(--spokenly-space-4)' : '0',
        borderBottom: title || subtitle ? `1px solid var(--spokenly-border-primary)` : 'none'
      }}
    >
      <div className="spokenly-card-header-content">
        {title && (
          <h3
            className="spokenly-card-title"
            style={{
              fontSize: 'var(--spokenly-text-lg)',
              fontWeight: 'var(--spokenly-font-semibold)',
              color: 'var(--spokenly-text-primary)',
              margin: '0 0 2px 0',
              lineHeight: 'var(--spokenly-leading-tight)'
            }}
          >
            {title}
          </h3>
        )}
        {subtitle && (
          <p
            className="spokenly-card-subtitle"
            style={{
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)',
              margin: '0',
              lineHeight: 'var(--spokenly-leading-normal)'
            }}
          >
            {subtitle}
          </p>
        )}
        {children}
      </div>
      {actions && (
        <div 
          className="spokenly-card-actions"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 'var(--spokenly-space-2)',
            marginLeft: 'var(--spokenly-space-4)'
          }}
        >
          {actions}
        </div>
      )}
    </div>
  );
};

// Card Body Component
export const SpokenlyCardBody: React.FC<CardBodyProps> = ({
  children,
  className = '',
  padding = 'none'
}) => {
  const getPadding = (size: string) => {
    switch (size) {
      case 'sm': return 'var(--spokenly-space-2)';
      case 'md': return 'var(--spokenly-space-4)';
      case 'lg': return 'var(--spokenly-space-6)';
      default: return '0';
    }
  };

  return (
    <div
      className={`spokenly-card-body ${className}`}
      style={{
        padding: getPadding(padding),
        flex: 1
      }}
    >
      {children}
    </div>
  );
};

// Card Footer Component
export const SpokenlyCardFooter: React.FC<CardFooterProps> = ({
  children,
  className = '',
  justify = 'end'
}) => {
  const getJustifyContent = (justify: string) => {
    switch (justify) {
      case 'start': return 'flex-start';
      case 'center': return 'center';
      case 'between': return 'space-between';
      default: return 'flex-end';
    }
  };

  return (
    <div
      className={`spokenly-card-footer ${className}`}
      style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: getJustifyContent(justify),
        gap: 'var(--spokenly-space-3)',
        marginTop: 'var(--spokenly-space-4)',
        paddingTop: 'var(--spokenly-space-4)',
        borderTop: `1px solid var(--spokenly-border-primary)`
      }}
    >
      {children}
    </div>
  );
};