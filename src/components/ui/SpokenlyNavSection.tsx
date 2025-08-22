/**
 * SpokenlyNavSection - Navigation section grouping
 * Provides grouped navigation items with optional titles
 */

import React from 'react';
import { motion } from 'framer-motion';
import { NavSectionProps } from './types';

export const SpokenlyNavSection: React.FC<NavSectionProps> = ({
  title,
  children,
  className = ''
}) => {
  const containerVariants = {
    hidden: { opacity: 0 },
    visible: {
      opacity: 1,
      transition: {
        staggerChildren: 0.05,
        delayChildren: 0.1
      }
    }
  };

  const itemVariants = {
    hidden: { opacity: 0, x: -10 },
    visible: { 
      opacity: 1, 
      x: 0,
      transition: { duration: 0.2, ease: [0.0, 0.0, 0.2, 1] }
    }
  };

  return (
    <motion.section
      className={`spokenly-nav-section ${className}`}
      variants={containerVariants}
      initial="hidden"
      animate="visible"
      style={{
        marginBottom: 'var(--spokenly-space-6)'
      }}
    >
      {/* Section Title */}
      {title && (
        <motion.h3
          className="spokenly-nav-section-title"
          variants={itemVariants}
          style={{
            fontSize: 'var(--spokenly-text-xs)',
            fontWeight: 'var(--spokenly-font-semibold)',
            color: 'var(--spokenly-text-tertiary)',
            textTransform: 'uppercase' as const,
            letterSpacing: '0.5px',
            margin: '0 0 var(--spokenly-space-3) 0',
            padding: '0 var(--spokenly-space-4)',
            lineHeight: 'var(--spokenly-leading-tight)'
          }}
        >
          {title}
        </motion.h3>
      )}

      {/* Section Content */}
      <motion.div
        className="spokenly-nav-section-content"
        variants={containerVariants}
        style={{
          display: 'flex',
          flexDirection: 'column',
          gap: '1px'
        }}
      >
        {React.Children.map(children, (child, index) => (
          <motion.div
            key={index}
            variants={itemVariants}
          >
            {child}
          </motion.div>
        ))}
      </motion.div>
    </motion.section>
  );
};