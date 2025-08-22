/**
 * Audio-Reactive Color Container Component
 * Applies dynamic color themes to child components based on audio characteristics
 */

import React, { useEffect, ReactNode } from 'react';
import { motion, MotionStyle } from 'framer-motion';
import { useAudioColorTheme, audioColorUtils } from '../hooks/useAudioColorTheme';

interface AudioColorContainerProps {
  children: ReactNode;
  isActive: boolean;
  sensitivity?: number;
  applyToCSS?: boolean;
  cssPrefix?: string;
  className?: string;
  style?: MotionStyle;
  animationType?: 'subtle' | 'dynamic' | 'reactive';
}

const AudioColorContainer: React.FC<AudioColorContainerProps> = ({
  children,
  isActive,
  sensitivity = 1.0,
  applyToCSS = true,
  cssPrefix = '--audio',
  className = '',
  style = {},
  animationType = 'dynamic',
}) => {
  const { theme, characteristics } = useAudioColorTheme(isActive, sensitivity);

  // Apply theme to CSS custom properties
  useEffect(() => {
    if (applyToCSS) {
      audioColorUtils.applyThemeToCSS(theme, cssPrefix);
    }
  }, [theme, applyToCSS, cssPrefix]);

  // Animation variants based on audio characteristics
  const getAnimationVariants = () => {
    if (!characteristics || animationType === 'subtle') {
      return {
        background: theme.background,
        transition: { duration: 0.3, ease: 'easeOut' },
      };
    }

    switch (animationType) {
      case 'dynamic':
        return {
          background: theme.gradient,
          scale: 1 + characteristics.energyLevel * 0.02,
          filter: `brightness(${1 + characteristics.energyLevel * 0.1})`,
          transition: { 
            duration: 0.2, 
            ease: 'easeOut',
            scale: { duration: 0.1 },
          },
        };
      
      case 'reactive':
        return {
          background: theme.gradient,
          scale: [
            1 + characteristics.energyLevel * 0.02,
            1 + characteristics.energyLevel * 0.05,
            1 + characteristics.energyLevel * 0.02,
          ],
          filter: `brightness(${1 + characteristics.energyLevel * 0.2}) saturate(${1 + characteristics.energyLevel * 0.3})`,
          boxShadow: `0 0 ${20 + characteristics.energyLevel * 30}px ${theme.pulseColor}`,
          transition: {
            duration: 0.6,
            repeat: characteristics.energyLevel > 0.3 ? Infinity : 0,
            ease: 'easeInOut',
          },
        };
      
      default:
        return {
          background: theme.background,
          transition: { duration: 0.3, ease: 'easeOut' },
        };
    }
  };

  const containerStyle: MotionStyle = {
    ...style,
    ...getAnimationVariants(),
  };

  return (
    <motion.div
      className={`audio-color-container ${className}`}
      style={containerStyle}
      animate={getAnimationVariants()}
    >
      {children}
      
      {/* Debug overlay for development */}
      {process.env.NODE_ENV === 'development' && characteristics && (
        <div className="audio-debug-overlay">
          <div style={{ fontSize: '10px', color: theme.textColor, opacity: 0.7 }}>
            <div>Energy: {characteristics.energyLevel.toFixed(2)}</div>
            <div>Freq: {characteristics.dominantFrequency.toFixed(0)}Hz</div>
            <div>Bass: {characteristics.bassLevel.toFixed(2)}</div>
            <div>Mid: {characteristics.midLevel.toFixed(2)}</div>
            <div>Treble: {characteristics.trebleLevel.toFixed(2)}</div>
          </div>
        </div>
      )}
    </motion.div>
  );
};

export default AudioColorContainer;