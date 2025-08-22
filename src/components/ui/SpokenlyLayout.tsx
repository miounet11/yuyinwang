/**
 * SpokenlyLayout - Main application layout container
 * Provides the overall structure with sidebar and content areas
 */

import React from 'react';
import { motion } from 'framer-motion';
import { LayoutProps } from './types';

export const SpokenlyLayout: React.FC<LayoutProps> = ({ 
  children, 
  className = '' 
}) => {
  return (
    <motion.div 
      className={`spokenly-layout ${className}`}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.3, ease: [0.0, 0.0, 0.2, 1] }}
      style={{
        display: 'flex',
        height: '100vh',
        backgroundColor: 'var(--spokenly-bg-app)',
        fontFamily: 'var(--spokenly-font-family)',
        overflow: 'hidden'
      }}
    >
      {children}
    </motion.div>
  );
};