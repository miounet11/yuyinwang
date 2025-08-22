/**
 * Advanced Micro-Interactions and Gesture Support
 * Enhanced user interaction system for Recording King voice input
 */

import React, { useState, useRef, useCallback } from 'react';
import { motion, useMotionValue, useSpring, PanInfo, useAnimation } from 'framer-motion';
import { useGesture } from '@use-gesture/react';

interface AdvancedInteractionProps {
  children: React.ReactNode;
  onTap?: () => void;
  onDoubleTap?: () => void;
  onLongPress?: () => void;
  onSwipe?: (direction: 'up' | 'down' | 'left' | 'right') => void;
  onPinch?: (scale: number) => void;
  onRotate?: (angle: number) => void;
  enableHaptic?: boolean;
  interactionType?: 'button' | 'container' | 'slider' | 'knob';
  audioReactive?: boolean;
  className?: string;
}

const AdvancedInteraction: React.FC<AdvancedInteractionProps> = ({
  children,
  onTap,
  onDoubleTap,
  onLongPress,
  onSwipe,
  onPinch,
  onRotate,
  enableHaptic = true,
  interactionType = 'button',
  audioReactive = false,
  className = '',
}) => {
  const [isPressed, setIsPressed] = useState(false);
  const [isHovered, setIsHovered] = useState(false);
  const [longPressTriggered, setLongPressTriggered] = useState(false);
  
  const controls = useAnimation();
  const longPressTimer = useRef<NodeJS.Timeout>();
  const tapCount = useRef(0);
  const lastTapTime = useRef(0);
  
  // Motion values for smooth interactions
  const scale = useMotionValue(1);
  const rotate = useMotionValue(0);
  const x = useMotionValue(0);
  const y = useMotionValue(0);
  
  // Spring configurations for different interaction types
  const springConfigs = {
    button: { stiffness: 400, damping: 30 },
    container: { stiffness: 300, damping: 25 },
    slider: { stiffness: 600, damping: 40 },
    knob: { stiffness: 500, damping: 35 },
  };
  
  const springScale = useSpring(scale, springConfigs[interactionType]);
  const springRotate = useSpring(rotate, springConfigs[interactionType]);
  const springX = useSpring(x, springConfigs[interactionType]);
  const springY = useSpring(y, springConfigs[interactionType]);

  // Haptic feedback utility
  const triggerHaptic = useCallback((type: 'light' | 'medium' | 'heavy' = 'light') => {
    if (!enableHaptic || typeof navigator === 'undefined') return;
    
    if ('vibrate' in navigator) {
      const patterns = {
        light: [10],
        medium: [20],
        heavy: [30, 10, 30],
      };
      navigator.vibrate(patterns[type]);
    }
  }, [enableHaptic]);

  // Enhanced gesture handlers
  const handleTapStart = useCallback(() => {
    setIsPressed(true);
    setLongPressTriggered(false);
    
    // Button press animation
    if (interactionType === 'button') {
      scale.set(0.95);
      triggerHaptic('light');
    }
    
    // Long press timer
    if (onLongPress) {
      longPressTimer.current = setTimeout(() => {
        setLongPressTriggered(true);
        onLongPress();
        triggerHaptic('heavy');
        controls.start({
          scale: [0.95, 1.05, 1],
          transition: { duration: 0.3 },
        });
      }, 500);
    }
  }, [interactionType, onLongPress, scale, triggerHaptic, controls]);

  const handleTapEnd = useCallback(() => {
    setIsPressed(false);
    scale.set(1);
    
    if (longPressTimer.current) {
      clearTimeout(longPressTimer.current);
    }
    
    if (!longPressTriggered) {
      const now = Date.now();
      const timeSinceLastTap = now - lastTapTime.current;
      
      if (timeSinceLastTap < 300) {
        // Double tap detected
        tapCount.current += 1;
        if (tapCount.current === 2) {
          onDoubleTap?.();
          triggerHaptic('medium');
          controls.start({
            scale: [1, 1.1, 1],
            rotate: [0, 5, -5, 0],
            transition: { duration: 0.4 },
          });
          tapCount.current = 0;
        }
      } else {
        // Single tap
        tapCount.current = 1;
        setTimeout(() => {
          if (tapCount.current === 1) {
            onTap?.();
            triggerHaptic('light');
          }
          tapCount.current = 0;
        }, 300);
      }
      
      lastTapTime.current = now;
    }
  }, [longPressTriggered, onTap, onDoubleTap, scale, triggerHaptic, controls]);

  // Advanced gesture recognition
  const gestureHandlers = useGesture({
    onDrag: ({ offset: [ox, oy], velocity: [vx, vy], direction: [dx, dy], last }) => {
      if (interactionType === 'slider' || interactionType === 'knob') {
        x.set(ox);
        y.set(oy);
      }
      
      // Swipe detection
      if (last && onSwipe) {
        const threshold = 50;
        const velocityThreshold = 0.5;
        
        if (Math.abs(ox) > threshold && Math.abs(vx) > velocityThreshold) {
          onSwipe(dx > 0 ? 'right' : 'left');
          triggerHaptic('medium');
        } else if (Math.abs(oy) > threshold && Math.abs(vy) > velocityThreshold) {
          onSwipe(dy > 0 ? 'down' : 'up');
          triggerHaptic('medium');
        }
      }
    },
    
    onPinch: ({ offset: [scale] }) => {
      if (onPinch) {
        onPinch(scale);
        triggerHaptic('light');
      }
    },
    
    onWheel: ({ offset: [, oy] }) => {
      if (interactionType === 'knob') {
        rotate.set(oy * 0.5);
        onRotate?.(oy * 0.5);
      }
    },
  });

  // Interaction-specific variants
  const getVariants = () => {
    const baseVariants = {
      idle: { 
        scale: 1, 
        rotate: 0,
        filter: 'brightness(1)',
        transition: { duration: 0.2 },
      },
      hover: { 
        scale: interactionType === 'button' ? 1.02 : 1.01,
        filter: 'brightness(1.05)',
        transition: { duration: 0.2 },
      },
      pressed: { 
        scale: interactionType === 'button' ? 0.95 : 0.98,
        filter: 'brightness(0.9)',
        transition: { duration: 0.1 },
      },
    };

    if (audioReactive) {
      baseVariants.hover.filter = 'brightness(1.1) saturate(1.1)';
      baseVariants.pressed.filter = 'brightness(0.85) saturate(1.2)';
    }

    return baseVariants;
  };

  // Audio-reactive micro-animations
  const audioReactiveProps = audioReactive ? {
    whileHover: {
      boxShadow: '0 0 20px var(--audio-pulse)',
      borderColor: 'var(--audio-primary)',
    },
    whileTap: {
      boxShadow: '0 0 30px var(--audio-accent)',
    },
  } : {};

  return (
    <motion.div
      className={`advanced-interaction ${interactionType} ${className}`}
      style={{
        scale: springScale,
        rotate: springRotate,
        x: springX,
        y: springY,
      }}
      variants={getVariants()}
      initial="idle"
      animate={isPressed ? 'pressed' : isHovered ? 'hover' : 'idle'}
      onHoverStart={() => setIsHovered(true)}
      onHoverEnd={() => setIsHovered(false)}
      onTapStart={handleTapStart}
      onTap={handleTapEnd}
      {...gestureHandlers()}
      {...audioReactiveProps}
    >
      {children}
    </motion.div>
  );
};

/**
 * Specialized Interaction Components
 */

// Enhanced Audio Button with waveform reaction
export const AudioReactiveButton: React.FC<{
  children: React.ReactNode;
  onClick?: () => void;
  variant?: 'primary' | 'secondary' | 'accent';
  size?: 'small' | 'medium' | 'large';
  className?: string;
}> = ({ children, onClick, variant = 'primary', size = 'medium', className = '' }) => {
  const [audioLevel, setAudioLevel] = useState(0);
  
  return (
    <AdvancedInteraction
      onTap={onClick}
      interactionType="button"
      audioReactive={true}
      className={`audio-reactive-button ${variant} ${size} ${className}`}
    >
      <motion.div
        className="button-content"
        style={{
          background: `var(--audio-${variant})`,
          boxShadow: `0 0 ${10 + audioLevel * 20}px var(--audio-pulse)`,
        }}
      >
        {children}
      </motion.div>
    </AdvancedInteraction>
  );
};

// Audio-reactive slider for settings
export const AudioReactiveSlider: React.FC<{
  value: number;
  onChange: (value: number) => void;
  min?: number;
  max?: number;
  step?: number;
  className?: string;
}> = ({ value, onChange, min = 0, max = 100, step = 1, className = '' }) => {
  const [isDragging, setIsDragging] = useState(false);
  
  const handleDrag = useCallback((info: PanInfo) => {
    const newValue = Math.min(max, Math.max(min, value + info.delta.x * 0.1));
    onChange(Math.round(newValue / step) * step);
  }, [value, onChange, min, max, step]);
  
  return (
    <motion.div className={`audio-reactive-slider ${className}`}>
      <motion.div 
        className="slider-track"
        style={{ background: 'var(--audio-background)' }}
      />
      <AdvancedInteraction
        interactionType="slider"
        audioReactive={true}
        className="slider-thumb"
      >
        <motion.div
          className="thumb"
          drag="x"
          dragConstraints={{ left: 0, right: 200 }}
          onDrag={(_, info) => handleDrag(info)}
          onDragStart={() => setIsDragging(true)}
          onDragEnd={() => setIsDragging(false)}
          style={{
            background: 'var(--audio-primary)',
            boxShadow: isDragging ? '0 0 15px var(--audio-pulse)' : '0 0 5px var(--audio-pulse)',
          }}
          whileDrag={{ scale: 1.2 }}
        />
      </AdvancedInteraction>
    </motion.div>
  );
};

// Voice input gesture zone
export const VoiceInputGestureZone: React.FC<{
  children: React.ReactNode;
  onVoiceStart: () => void;
  onVoiceEnd: () => void;
  onSwipeUp?: () => void;
  onSwipeDown?: () => void;
  className?: string;
}> = ({ children, onVoiceStart, onVoiceEnd, onSwipeUp, onSwipeDown, className = '' }) => {
  return (
    <AdvancedInteraction
      onLongPress={onVoiceStart}
      onSwipe={(direction) => {
        if (direction === 'up') onSwipeUp?.();
        if (direction === 'down') onSwipeDown?.();
      }}
      interactionType="container"
      audioReactive={true}
      className={`voice-input-gesture-zone ${className}`}
    >
      <motion.div
        className="gesture-zone-content"
        whileHover={{
          background: 'var(--audio-gradient)',
          scale: 1.01,
        }}
        whileTap={{
          background: 'var(--audio-primary)',
          scale: 0.99,
        }}
      >
        {children}
      </motion.div>
    </AdvancedInteraction>
  );
};

export default AdvancedInteraction;