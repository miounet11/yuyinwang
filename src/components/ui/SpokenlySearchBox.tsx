/**
 * SpokenlySearchBox - Search input component
 * Provides search functionality with clear button and debouncing
 */

import React, { useState, useEffect, useRef } from 'react';
import { motion } from 'framer-motion';
import { SearchBoxProps } from './types';

export const SpokenlySearchBox: React.FC<SearchBoxProps> = ({
  onSearch,
  onClear,
  showClearButton = true,
  debounceMs = 300,
  placeholder = 'Search...',
  className = '',
  ...inputProps
}) => {
  const [value, setValue] = useState(inputProps.value || inputProps.defaultValue || '');
  const [isFocused, setIsFocused] = useState(false);
  const debounceTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Debounced search effect
  useEffect(() => {
    if (debounceTimeoutRef.current) {
      clearTimeout(debounceTimeoutRef.current);
    }

    debounceTimeoutRef.current = setTimeout(() => {
      if (onSearch) {
        onSearch(value);
      }
    }, debounceMs);

    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
    };
  }, [value, debounceMs, onSearch]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setValue(newValue);
    inputProps.onChange?.(e);
  };

  const handleClear = () => {
    setValue('');
    onClear?.();
    if (inputProps.onChange) {
      const event = {
        target: { value: '' }
      } as React.ChangeEvent<HTMLInputElement>;
      inputProps.onChange(event);
    }
  };

  const handleFocus = (e: React.FocusEvent<HTMLInputElement>) => {
    setIsFocused(true);
    inputProps.onFocus?.(e);
  };

  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    setIsFocused(false);
    inputProps.onBlur?.(e);
  };

  const searchIcon = (
    <svg width="18" height="18" viewBox="0 0 18 18" fill="currentColor">
      <path d="M12.5 11h-.79l-.28-.27C12.41 9.59 13 8.11 13 6.5 13 2.91 10.09 0 6.5 0S0 2.91 0 6.5 2.91 13 6.5 13c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L17.49 16l-4.99-5zm-6 0C4.01 11 2 8.99 2 6.5S4.01 2 6.5 2 11 4.01 11 6.5 8.99 11 6.5 11z" fill="none" stroke="currentColor" strokeWidth="1.5"/>
    </svg>
  );

  const clearIcon = (
    <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
      <path d="M11 3L3 11m8 0L3 3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" fill="none"/>
    </svg>
  );

  return (
    <div 
      className={`spokenly-search-box ${className}`}
      style={{
        position: 'relative',
        width: inputProps.fullWidth ? '100%' : 'auto'
      }}
    >
      {/* Search Input Container */}
      <div
        className="spokenly-search-container"
        style={{
          position: 'relative',
          display: 'flex',
          alignItems: 'center'
        }}
      >
        {/* Search Input */}
        <motion.input
          type="text"
          className="spokenly-search-input"
          value={value}
          onChange={handleChange}
          onFocus={handleFocus}
          onBlur={handleBlur}
          placeholder={placeholder}
          style={{
            width: '100%',
            fontFamily: 'var(--spokenly-font-family)',
            fontSize: 'var(--spokenly-text-base)',
            fontWeight: 'var(--spokenly-font-normal)',
            color: 'var(--spokenly-text-primary)',
            backgroundColor: 'var(--spokenly-bg-content)',
            border: `1px solid ${isFocused ? 'var(--spokenly-primary)' : 'var(--spokenly-border-primary)'}`,
            borderRadius: 'var(--spokenly-radius-full)',
            padding: '10px 20px 10px 44px',
            paddingRight: value && showClearButton ? '44px' : '20px',
            outline: 'none',
            transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
            minHeight: '40px'
          }}
          whileFocus={{
            boxShadow: 'var(--spokenly-shadow-focus)'
          }}
          {...inputProps}
        />

        {/* Search Icon */}
        <div
          className="spokenly-search-icon"
          style={{
            position: 'absolute',
            left: '14px',
            top: '50%',
            transform: 'translateY(-50%)',
            color: isFocused ? 'var(--spokenly-primary)' : 'var(--spokenly-text-tertiary)',
            transition: 'color var(--spokenly-duration-fast) var(--spokenly-ease-out)',
            pointerEvents: 'none',
            display: 'flex',
            alignItems: 'center'
          }}
        >
          {searchIcon}
        </div>

        {/* Clear Button */}
        {value && showClearButton && (
          <motion.button
            className="spokenly-search-clear"
            onClick={handleClear}
            style={{
              position: 'absolute',
              right: '12px',
              top: '50%',
              transform: 'translateY(-50%)',
              width: '20px',
              height: '20px',
              border: 'none',
              background: 'none',
              borderRadius: '50%',
              cursor: 'pointer',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              color: 'var(--spokenly-text-tertiary)',
              transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)'
            }}
            initial={{ opacity: 0, scale: 0 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0 }}
            whileHover={{
              backgroundColor: 'var(--spokenly-bg-hover)',
              color: 'var(--spokenly-text-primary)',
              scale: 1.1
            }}
            whileTap={{ scale: 0.9 }}
          >
            {clearIcon}
          </motion.button>
        )}
      </div>

      {/* Search Suggestions/Results (if needed in future) */}
      {/* This section can be extended to show search suggestions or results */}
    </div>
  );
};