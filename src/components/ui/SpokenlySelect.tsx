/**
 * SpokenlySelect - Select dropdown component
 * Provides dropdown selection with search and customization options
 */

import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import * as Select from '@radix-ui/react-dropdown-menu';
import { SelectProps } from './types';

export const SpokenlySelect: React.FC<SelectProps> = ({
  value,
  defaultValue,
  onValueChange,
  placeholder = 'Select an option...',
  options = [],
  disabled = false,
  label,
  errorText,
  size = 'md',
  fullWidth = false,
  className = ''
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [selectedValue, setSelectedValue] = useState(value || defaultValue || '');

  const hasError = !!errorText;
  const selectedOption = options.find(option => option.value === selectedValue);

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

  const triggerStyles = {
    width: fullWidth ? '100%' : 'auto',
    minWidth: '200px',
    fontFamily: 'var(--spokenly-font-family)',
    fontWeight: 'var(--spokenly-font-normal)',
    color: disabled ? 'var(--spokenly-text-tertiary)' : selectedValue ? 'var(--spokenly-text-primary)' : 'var(--spokenly-text-secondary)',
    backgroundColor: disabled ? 'var(--spokenly-bg-hover)' : 'var(--spokenly-bg-content)',
    border: `1px solid ${hasError ? 'var(--spokenly-error)' : isOpen ? 'var(--spokenly-primary)' : 'var(--spokenly-border-primary)'}`,
    borderRadius: 'var(--spokenly-radius-base)',
    cursor: disabled ? 'not-allowed' : 'pointer',
    outline: 'none',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
    ...sizeStyles[size]
  };

  const handleValueChange = (newValue: string) => {
    setSelectedValue(newValue);
    onValueChange?.(newValue);
    setIsOpen(false);
  };

  return (
    <div 
      className={`spokenly-select-container ${className}`}
      style={{
        width: fullWidth ? '100%' : 'auto',
        position: 'relative'
      }}
    >
      {/* Label */}
      {label && (
        <motion.label
          className="spokenly-select-label"
          initial={false}
          animate={{
            color: hasError ? 'var(--spokenly-error)' : isOpen ? 'var(--spokenly-primary)' : 'var(--spokenly-text-secondary)'
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

      <Select.Root onOpenChange={setIsOpen}>
        {/* Trigger */}
        <Select.Trigger asChild>
          <motion.button
            className="spokenly-select-trigger"
            style={triggerStyles}
            disabled={disabled}
            whileFocus={{
              boxShadow: hasError ? '0 0 0 3px rgba(255, 59, 48, 0.15)' : 'var(--spokenly-shadow-focus)'
            }}
          >
            <span className="spokenly-select-value">
              {selectedOption ? (
                <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--spokenly-space-2)' }}>
                  {selectedOption.icon && (
                    <span style={{ display: 'flex', alignItems: 'center', fontSize: '16px' }}>
                      {selectedOption.icon}
                    </span>
                  )}
                  {selectedOption.label}
                </div>
              ) : (
                placeholder
              )}
            </span>
            
            {/* Dropdown Arrow */}
            <motion.span
              className="spokenly-select-arrow"
              animate={{ rotate: isOpen ? 180 : 0 }}
              transition={{ duration: 0.2 }}
              style={{
                display: 'flex',
                alignItems: 'center',
                color: 'var(--spokenly-text-tertiary)',
                marginLeft: 'var(--spokenly-space-2)'
              }}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                <path d="M4 6l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
              </svg>
            </motion.span>
          </motion.button>
        </Select.Trigger>

        {/* Content */}
        <Select.Portal>
          <AnimatePresence>
            {isOpen && (
              <Select.Content asChild>
                <motion.div
                  className="spokenly-select-content"
                  initial={{ opacity: 0, y: -10, scale: 0.95 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: -10, scale: 0.95 }}
                  transition={{ duration: 0.15, ease: [0.0, 0.0, 0.2, 1] }}
                  style={{
                    backgroundColor: 'var(--spokenly-bg-content)',
                    border: '1px solid var(--spokenly-border-primary)',
                    borderRadius: 'var(--spokenly-radius-base)',
                    boxShadow: 'var(--spokenly-shadow-lg)',
                    padding: 'var(--spokenly-space-2)',
                    minWidth: '200px',
                    maxHeight: '300px',
                    overflowY: 'auto',
                    zIndex: 'var(--spokenly-z-dropdown)'
                  }}
                >
                  <Select.Viewport>
                    {options.map((option) => (
                      <Select.Item
                        key={option.value}
                        value={option.value}
                        disabled={option.disabled}
                        asChild
                      >
                        <motion.div
                          className="spokenly-select-item"
                          style={{
                            padding: '8px 12px',
                            borderRadius: 'var(--spokenly-radius-sm)',
                            cursor: option.disabled ? 'not-allowed' : 'pointer',
                            display: 'flex',
                            alignItems: 'center',
                            gap: 'var(--spokenly-space-2)',
                            fontSize: 'var(--spokenly-text-sm)',
                            color: option.disabled ? 'var(--spokenly-text-tertiary)' : 'var(--spokenly-text-primary)',
                            opacity: option.disabled ? 0.5 : 1
                          }}
                          whileHover={option.disabled ? {} : {
                            backgroundColor: 'var(--spokenly-bg-hover)'
                          }}
                          onClick={() => !option.disabled && handleValueChange(option.value)}
                        >
                          {option.icon && (
                            <span style={{ display: 'flex', alignItems: 'center', fontSize: '16px' }}>
                              {option.icon}
                            </span>
                          )}
                          <Select.ItemText>{option.label}</Select.ItemText>
                          {selectedValue === option.value && (
                            <span style={{ marginLeft: 'auto', color: 'var(--spokenly-primary)' }}>
                              <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
                                <path d="M13 4L6 11l-3-3" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
                              </svg>
                            </span>
                          )}
                        </motion.div>
                      </Select.Item>
                    ))}
                  </Select.Viewport>
                </motion.div>
              </Select.Content>
            )}
          </AnimatePresence>
        </Select.Portal>
      </Select.Root>

      {/* Error Text */}
      {errorText && (
        <motion.p
          className="spokenly-select-error"
          initial={{ opacity: 0, y: -5 }}
          animate={{ opacity: 1, y: 0 }}
          style={{
            fontSize: 'var(--spokenly-text-sm)',
            color: 'var(--spokenly-error)',
            margin: '4px 0 0 0',
            lineHeight: 'var(--spokenly-leading-normal)'
          }}
        >
          {errorText}
        </motion.p>
      )}
    </div>
  );
};