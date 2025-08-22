/**
 * SpokemlySidebar - Left navigation sidebar
 * Provides navigation structure with collapsible functionality
 */

import React from 'react';
import { motion } from 'framer-motion';
import { SidebarProps } from './types';

export const SpokemlySidebar: React.FC<SidebarProps> = ({
  children,
  className = '',
  isCollapsed = false,
  onToggle,
  width = 250
}) => {
  const sidebarVariants = {
    expanded: { width: width },
    collapsed: { width: 60 }
  };

  return (
    <motion.aside
      className={`spokenly-sidebar ${className}`}
      variants={sidebarVariants}
      animate={isCollapsed ? 'collapsed' : 'expanded'}
      transition={{ 
        duration: 0.25,
        ease: [0.4, 0.0, 0.2, 1]
      }}
      style={{
        backgroundColor: 'var(--spokenly-bg-sidebar)',
        borderRight: '1px solid var(--spokenly-border-primary)',
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        overflow: 'hidden',
        zIndex: 'var(--spokenly-z-base)'
      }}
    >
      {/* Toggle Button */}
      {onToggle && (
        <motion.button
          onClick={onToggle}
          className="spokenly-sidebar-toggle"
          whileHover={{ scale: 1.05 }}
          whileTap={{ scale: 0.95 }}
          style={{
            position: 'absolute',
            top: 'var(--spokenly-space-4)',
            right: 'var(--spokenly-space-2)',
            width: '24px',
            height: '24px',
            border: 'none',
            background: 'transparent',
            borderRadius: 'var(--spokenly-radius-sm)',
            cursor: 'pointer',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'var(--spokenly-text-secondary)',
            transition: 'color var(--spokenly-duration-fast) var(--spokenly-ease-out)'
          }}
        >
          <motion.div
            animate={{ rotate: isCollapsed ? 180 : 0 }}
            transition={{ duration: 0.2 }}
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M10.5 3.5L6 8l4.5 4.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
            </svg>
          </motion.div>
        </motion.button>
      )}

      {/* Sidebar Content */}
      <div 
        className="spokenly-sidebar-content"
        style={{
          flex: 1,
          paddingTop: onToggle ? 'var(--spokenly-space-12)' : 'var(--spokenly-space-6)',
          paddingBottom: 'var(--spokenly-space-6)',
          paddingLeft: 'var(--spokenly-space-4)',
          paddingRight: 'var(--spokenly-space-4)',
          overflowY: 'auto',
          overflowX: 'hidden'
        }}
      >
        {children}
      </div>
    </motion.aside>
  );
};