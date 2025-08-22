/**
 * SpokenlyContent - Main content area
 * Provides the main content container with proper spacing
 */

import React from 'react';
import { motion } from 'framer-motion';
import { ContentProps } from './types';

export const SpokenlyContent: React.FC<ContentProps> = ({
  children,
  className = '',
  sidebarWidth = 250,
  padding = 'md'
}) => {
  const getPadding = (size: string) => {
    switch (size) {
      case 'sm': return 'var(--spokenly-space-4)';
      case 'lg': return 'var(--spokenly-space-8)';
      default: return 'var(--spokenly-content-padding)';
    }
  };

  return (
    <motion.main
      className={`spokenly-content ${className}`}
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ 
        duration: 0.3,
        delay: 0.1,
        ease: [0.0, 0.0, 0.2, 1]
      }}
      style={{
        flex: 1,
        backgroundColor: 'var(--spokenly-bg-content)',
        padding: getPadding(padding),
        overflowY: 'auto',
        overflowX: 'hidden',
        height: '100vh',
        position: 'relative'
      }}
    >
      <div
        className="spokenly-content-inner"
        style={{
          maxWidth: '100%',
          height: '100%'
        }}
      >
        {children}
      </div>
    </motion.main>
  );
};