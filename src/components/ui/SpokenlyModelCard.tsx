/**
 * SpokenlyModelCard - AI Model selection card
 * Provides model selection with status indicators, tags, and actions
 */

import React from 'react';
import { motion } from 'framer-motion';
import { ModelCardProps } from './types';
import { SpokenlyCard, SpokenlyCardHeader, SpokenlyCardBody, SpokenlyCardFooter } from './SpokenlyCard';
import { SpokenlyTag } from './SpokenlyTag';

export const SpokenlyModelCard: React.FC<ModelCardProps> = ({
  title,
  description,
  provider,
  status,
  isSelected = false,
  onSelect,
  tags = [],
  actions,
  image,
  pricing,
  className = ''
}) => {
  // Status indicator styles
  const getStatusStyles = () => {
    const baseStyles = {
      width: '8px',
      height: '8px',
      borderRadius: '50%',
      display: 'inline-block',
      marginRight: 'var(--spokenly-space-2)'
    };

    switch (status.type) {
      case 'online':
        return {
          ...baseStyles,
          backgroundColor: 'var(--spokenly-success)',
          boxShadow: '0 0 4px rgba(52, 199, 89, 0.4)'
        };
      case 'offline':
        return {
          ...baseStyles,
          backgroundColor: 'var(--spokenly-text-tertiary)'
        };
      case 'loading':
        return {
          ...baseStyles,
          backgroundColor: 'var(--spokenly-warning)'
        };
      case 'error':
        return {
          ...baseStyles,
          backgroundColor: 'var(--spokenly-error)',
          boxShadow: '0 0 4px rgba(255, 59, 48, 0.4)'
        };
      default:
        return baseStyles;
    }
  };

  const getStatusText = () => {
    switch (status.type) {
      case 'online':
        return 'Online';
      case 'offline':
        return 'Offline';
      case 'loading':
        return 'Loading...';
      case 'error':
        return 'Error';
      default:
        return 'Unknown';
    }
  };

  const getStatusColor = () => {
    switch (status.type) {
      case 'online':
        return 'var(--spokenly-success)';
      case 'offline':
        return 'var(--spokenly-text-tertiary)';
      case 'loading':
        return 'var(--spokenly-warning)';
      case 'error':
        return 'var(--spokenly-error)';
      default:
        return 'var(--spokenly-text-secondary)';
    }
  };

  return (
    <SpokenlyCard
      className={`spokenly-model-card ${className}`}
      hover={!!onSelect}
      selected={isSelected}
      onClick={onSelect}
      style={{
        cursor: onSelect ? 'pointer' : 'default',
        position: 'relative'
      }}
    >
      {/* Selection Indicator */}
      {isSelected && (
        <motion.div
          className="spokenly-model-card-selection"
          initial={{ scale: 0, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          style={{
            position: 'absolute',
            top: 'var(--spokenly-space-4)',
            right: 'var(--spokenly-space-4)',
            width: '20px',
            height: '20px',
            borderRadius: '50%',
            backgroundColor: 'var(--spokenly-primary)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'white',
            fontSize: '12px',
            zIndex: 1
          }}
        >
          ✓
        </motion.div>
      )}

      <SpokenlyCardHeader
        title={
          <div style={{ display: 'flex', alignItems: 'center' }}>
            {image && (
              <img
                src={image}
                alt={`${title} logo`}
                style={{
                  width: '24px',
                  height: '24px',
                  borderRadius: 'var(--spokenly-radius-sm)',
                  marginRight: 'var(--spokenly-space-3)',
                  objectFit: 'cover'
                }}
              />
            )}
            <span>{title}</span>
          </div>
        }
        subtitle={provider}
        actions={actions}
      />

      <SpokenlyCardBody>
        {/* Status */}
        <div 
          className="spokenly-model-status"
          style={{
            display: 'flex',
            alignItems: 'center',
            marginBottom: 'var(--spokenly-space-3)',
            fontSize: 'var(--spokenly-text-sm)'
          }}
        >
          {/* Status Indicator */}
          {status.type === 'loading' ? (
            <motion.div
              style={{
                ...getStatusStyles(),
                marginRight: 'var(--spokenly-space-2)'
              }}
              animate={{ 
                scale: [1, 1.2, 1],
                opacity: [0.6, 1, 0.6]
              }}
              transition={{
                duration: 1.5,
                repeat: Infinity,
                ease: 'easeInOut'
              }}
            />
          ) : (
            <span style={getStatusStyles()} />
          )}
          
          <span style={{ color: getStatusColor(), fontWeight: 'var(--spokenly-font-medium)' }}>
            {getStatusText()}
          </span>
          
          {status.message && (
            <span style={{ 
              color: 'var(--spokenly-text-secondary)', 
              marginLeft: 'var(--spokenly-space-2)' 
            }}>
              • {status.message}
            </span>
          )}
        </div>

        {/* Description */}
        {description && (
          <p
            className="spokenly-model-description"
            style={{
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)',
              lineHeight: 'var(--spokenly-leading-relaxed)',
              margin: '0 0 var(--spokenly-space-4) 0'
            }}
          >
            {description}
          </p>
        )}

        {/* Tags */}
        {tags.length > 0 && (
          <div
            className="spokenly-model-tags"
            style={{
              display: 'flex',
              flexWrap: 'wrap',
              gap: 'var(--spokenly-space-2)',
              marginBottom: 'var(--spokenly-space-4)'
            }}
          >
            {tags.map((tag, index) => (
              <SpokenlyTag key={index} size="sm" variant="info">
                {tag}
              </SpokenlyTag>
            ))}
          </div>
        )}
      </SpokenlyCardBody>

      {/* Pricing */}
      {pricing && (
        <SpokenlyCardFooter justify="between">
          <span
            style={{
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)'
            }}
          >
            {pricing}
          </span>
        </SpokenlyCardFooter>
      )}

      {/* Hover Effect Overlay */}
      {onSelect && (
        <motion.div
          className="spokenly-model-card-overlay"
          style={{
            position: 'absolute',
            inset: 0,
            borderRadius: 'inherit',
            background: 'linear-gradient(135deg, rgba(0, 122, 255, 0.05), rgba(0, 122, 255, 0.02))',
            opacity: isSelected ? 1 : 0,
            pointerEvents: 'none',
            transition: 'opacity var(--spokenly-duration-fast) var(--spokenly-ease-out)'
          }}
          animate={{ opacity: isSelected ? 1 : 0 }}
        />
      )}
    </SpokenlyCard>
  );
};