/**
 * SpokenlySwitch - Toggle switch component
 * Provides iOS-style toggle switches with smooth animations
 */

import React from 'react';
import { motion } from 'framer-motion';
import * as Switch from '@radix-ui/react-switch';
import { SwitchProps } from './types';

export const SpokenlySwitch: React.FC<SwitchProps> = ({
  checked,
  defaultChecked,
  onCheckedChange,
  disabled = false,
  label,
  description,
  size = 'md',
  className = ''
}) => {
  // Size mappings
  const sizeStyles = {
    sm: {
      width: '36px',
      height: '20px',
      thumbSize: '16px',
      thumbTranslate: '14px'
    },
    md: {
      width: '44px',
      height: '24px',
      thumbSize: '20px',
      thumbTranslate: '18px'
    },
    lg: {
      width: '52px',
      height: '28px',
      thumbSize: '24px',
      thumbTranslate: '22px'
    }
  };

  const { width, height, thumbSize, thumbTranslate } = sizeStyles[size];

  return (
    <div 
      className={`spokenly-switch-container ${className}`}
      style={{
        display: 'flex',
        alignItems: label || description ? 'flex-start' : 'center',
        gap: 'var(--spokenly-space-3)',
        opacity: disabled ? 0.6 : 1,
        cursor: disabled ? 'not-allowed' : 'pointer'
      }}
    >
      {/* Switch Component */}
      <Switch.Root
        className="spokenly-switch-root"
        checked={checked}
        defaultChecked={defaultChecked}
        onCheckedChange={disabled ? undefined : onCheckedChange}
        disabled={disabled}
        style={{
          width,
          height,
          backgroundColor: 'var(--spokenly-border-secondary)',
          borderRadius: height,
          position: 'relative',
          border: 'none',
          cursor: disabled ? 'not-allowed' : 'pointer',
          transition: 'background-color var(--spokenly-duration-fast) var(--spokenly-ease-out)',
          '&[data-state="checked"]': {
            backgroundColor: 'var(--spokenly-primary)'
          }
        }}
        data-state={checked ? 'checked' : 'unchecked'}
      >
        <Switch.Thumb asChild>
          <motion.div
            className="spokenly-switch-thumb"
            style={{
              display: 'block',
              width: thumbSize,
              height: thumbSize,
              backgroundColor: 'white',
              borderRadius: '50%',
              boxShadow: 'var(--spokenly-shadow-sm)',
              position: 'absolute',
              top: '2px',
              left: '2px',
              willChange: 'transform'
            }}
            animate={{
              transform: checked ? `translateX(${thumbTranslate})` : 'translateX(0px)'
            }}
            transition={{
              type: 'spring',
              stiffness: 500,
              damping: 30,
              mass: 1
            }}
            whileTap={{ scale: 0.9 }}
          />
        </Switch.Thumb>

        {/* Animated Background */}
        <motion.div
          className="spokenly-switch-bg"
          style={{
            position: 'absolute',
            inset: 0,
            borderRadius: height,
            pointerEvents: 'none'
          }}
          animate={{
            backgroundColor: checked 
              ? disabled 
                ? 'var(--spokenly-border-secondary)' 
                : 'var(--spokenly-primary)'
              : 'var(--spokenly-border-secondary)'
          }}
          transition={{ duration: 0.2, ease: [0.0, 0.0, 0.2, 1] }}
        />
      </Switch.Root>

      {/* Label and Description */}
      {(label || description) && (
        <div className="spokenly-switch-content">
          {label && (
            <label
              className="spokenly-switch-label"
              style={{
                fontSize: 'var(--spokenly-text-base)',
                fontWeight: 'var(--spokenly-font-medium)',
                color: disabled ? 'var(--spokenly-text-tertiary)' : 'var(--spokenly-text-primary)',
                lineHeight: 'var(--spokenly-leading-tight)',
                margin: 0,
                cursor: disabled ? 'not-allowed' : 'pointer'
              }}
            >
              {label}
            </label>
          )}
          {description && (
            <p
              className="spokenly-switch-description"
              style={{
                fontSize: 'var(--spokenly-text-sm)',
                color: disabled ? 'var(--spokenly-text-tertiary)' : 'var(--spokenly-text-secondary)',
                lineHeight: 'var(--spokenly-leading-normal)',
                margin: label ? '2px 0 0 0' : 0
              }}
            >
              {description}
            </p>
          )}
        </div>
      )}
    </div>
  );
};