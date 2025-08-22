/**
 * Motion Components Library for Recording King
 * Provides reusable motion-enhanced components with consistent animations
 */

import React, { forwardRef } from 'react';
import { motion, MotionProps, HTMLMotionProps } from 'framer-motion';
import { 
  VOICE_INPUT_VARIANTS, 
  SPRING_CONFIGS, 
  getMotionPreferences,
  createReducedMotionVariant,
  getOptimizedMotionProps 
} from '../../utils/motionUtils';

// =============================================================================
// BASE MOTION WRAPPER COMPONENTS
// =============================================================================

interface BaseMotionProps extends HTMLMotionProps<'div'> {
  reduceMotion?: boolean;
  optimized?: boolean;
}

/**
 * Base motion wrapper with accessibility and performance optimizations
 */
export const MotionWrapper = forwardRef<HTMLDivElement, BaseMotionProps>(
  ({ children, reduceMotion, optimized = true, variants, ...props }, ref) => {
    const { shouldAnimate } = getMotionPreferences();
    const finalVariants = reduceMotion || !shouldAnimate 
      ? createReducedMotionVariant(variants || {})
      : variants;

    const optimizedProps = optimized ? getOptimizedMotionProps() : {};

    return (
      <motion.div
        ref={ref}
        variants={finalVariants}
        {...optimizedProps}
        {...props}
      >
        {children}
      </motion.div>
    );
  }
);

MotionWrapper.displayName = 'MotionWrapper';

// =============================================================================
// VOICE INPUT SPECIFIC COMPONENTS
// =============================================================================

interface VoiceInputContainerProps extends BaseMotionProps {
  state: 'idle' | 'listening' | 'processing' | 'injecting';
}

/**
 * Animated container for voice input with state-based animations
 */
export const MotionVoiceInputContainer = forwardRef<HTMLDivElement, VoiceInputContainerProps>(
  ({ state, children, className, ...props }, ref) => {
    const containerVariants = {
      ...VOICE_INPUT_VARIANTS.container,
      [state]: VOICE_INPUT_VARIANTS[state],
    };

    return (
      <MotionWrapper
        ref={ref}
        className={className}
        variants={containerVariants}
        initial="hidden"
        animate={['visible', state]}
        exit="exit"
        transition={SPRING_CONFIGS.bouncy}
        {...props}
      >
        {children}
      </MotionWrapper>
    );
  }
);

MotionVoiceInputContainer.displayName = 'MotionVoiceInputContainer';

// =============================================================================
// BUTTON COMPONENTS
// =============================================================================

interface MotionButtonProps extends HTMLMotionProps<'button'> {
  variant?: 'default' | 'close';
  isProcessing?: boolean;
}

/**
 * Animated button with hover and tap states
 */
export const MotionButton = forwardRef<HTMLButtonElement, MotionButtonProps>(
  ({ variant = 'default', isProcessing = false, children, disabled, ...props }, ref) => {
    const buttonVariants = variant === 'close' 
      ? VOICE_INPUT_VARIANTS.closeButton 
      : VOICE_INPUT_VARIANTS.button;

    return (
      <motion.button
        ref={ref}
        variants={buttonVariants}
        initial="rest"
        whileHover={!disabled && !isProcessing ? "hover" : undefined}
        whileTap={!disabled && !isProcessing ? "tap" : undefined}
        disabled={disabled || isProcessing}
        {...getOptimizedMotionProps(['transform', 'box-shadow'])}
        {...props}
      >
        {children}
      </motion.button>
    );
  }
);

MotionButton.displayName = 'MotionButton';

// =============================================================================
// WAVEFORM COMPONENTS
// =============================================================================

interface WaveformBarProps extends HTMLMotionProps<'div'> {
  isActive: boolean;
  audioLevel?: number;
  index?: number;
}

/**
 * Individual animated waveform bar
 */
export const MotionWaveformBar = forwardRef<HTMLDivElement, WaveformBarProps>(
  ({ isActive, audioLevel = 0, index = 0, style, ...props }, ref) => {
    const height = isActive 
      ? 8 + audioLevel * 12 + Math.random() * 4
      : 8;

    const animationDelay = index * 0.05;

    return (
      <motion.div
        ref={ref}
        variants={VOICE_INPUT_VARIANTS.waveformBar}
        initial="idle"
        animate={isActive ? "active" : "idle"}
        style={{
          height: `${height}px`,
          animationDelay: `${animationDelay}s`,
          ...style,
        }}
        transition={{
          ...SPRING_CONFIGS.gentle,
          delay: animationDelay,
        }}
        {...getOptimizedMotionProps(['transform', 'height'])}
        {...props}
      />
    );
  }
);

MotionWaveformBar.displayName = 'MotionWaveformBar';

interface WaveformContainerProps extends BaseMotionProps {
  isActive: boolean;
  audioLevel: number;
  barCount?: number;
}

/**
 * Container for animated waveform bars
 */
export const MotionWaveformContainer = forwardRef<HTMLDivElement, WaveformContainerProps>(
  ({ isActive, audioLevel, barCount = 10, className, ...props }, ref) => {
    return (
      <MotionWrapper
        ref={ref}
        className={`waveform-container ${className || ''}`}
        initial="hidden"
        animate="visible"
        variants={VOICE_INPUT_VARIANTS.container}
        {...props}
      >
        <div className="waveform-bars">
          {Array.from({ length: barCount }, (_, i) => (
            <MotionWaveformBar
              key={i}
              isActive={isActive}
              audioLevel={audioLevel}
              index={i}
              className="waveform-bar"
            />
          ))}
        </div>
      </MotionWrapper>
    );
  }
);

MotionWaveformContainer.displayName = 'MotionWaveformContainer';

// =============================================================================
// TEXT COMPONENTS
// =============================================================================

interface MotionTextProps extends HTMLMotionProps<'div'> {
  isVisible?: boolean;
  typewriter?: boolean;
}

/**
 * Animated text with fade and slide effects
 */
export const MotionText = forwardRef<HTMLDivElement, MotionTextProps>(
  ({ isVisible = true, typewriter = false, children, ...props }, ref) => {
    const textVariants = typewriter 
      ? { ...VOICE_INPUT_VARIANTS.text, typing: VOICE_INPUT_VARIANTS.text.typing }
      : VOICE_INPUT_VARIANTS.text;

    return (
      <motion.div
        ref={ref}
        variants={textVariants}
        initial="hidden"
        animate={isVisible ? (typewriter ? "typing" : "visible") : "hidden"}
        {...getOptimizedMotionProps(['transform', 'opacity'])}
        {...props}
      >
        {children}
      </motion.div>
    );
  }
);

MotionText.displayName = 'MotionText';

// =============================================================================
// STATUS COMPONENTS
// =============================================================================

interface ProcessingSpinnerProps extends HTMLMotionProps<'div'> {
  size?: number;
}

/**
 * Animated processing spinner
 */
export const MotionProcessingSpinner = forwardRef<HTMLDivElement, ProcessingSpinnerProps>(
  ({ size = 12, style, ...props }, ref) => {
    return (
      <motion.div
        ref={ref}
        variants={VOICE_INPUT_VARIANTS.spinner}
        animate="spin"
        style={{
          width: size,
          height: size,
          border: '1px solid rgba(255, 255, 255, 0.1)',
          borderTopColor: 'rgba(147, 51, 234, 0.8)',
          borderRadius: '50%',
          ...style,
        }}
        {...getOptimizedMotionProps(['transform'])}
        {...props}
      />
    );
  }
);

MotionProcessingSpinner.displayName = 'MotionProcessingSpinner';

interface SuccessCheckProps extends HTMLMotionProps<'svg'> {
  size?: number;
  strokeWidth?: number;
}

/**
 * Animated success checkmark
 */
export const MotionSuccessCheck = forwardRef<SVGSVGElement, SuccessCheckProps>(
  ({ size = 12, strokeWidth = 2, ...props }, ref) => {
    return (
      <motion.svg
        ref={ref}
        width={size}
        height={size}
        viewBox="0 0 24 24"
        fill="none"
        {...props}
      >
        <motion.path
          d="M5 12l5 5L20 7"
          stroke="currentColor"
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeLinejoin="round"
          variants={VOICE_INPUT_VARIANTS.successCheck}
          initial="hidden"
          animate="visible"
        />
      </motion.svg>
    );
  }
);

MotionSuccessCheck.displayName = 'MotionSuccessCheck';

// =============================================================================
// ICON COMPONENTS
// =============================================================================

interface AppIconWrapperProps extends HTMLMotionProps<'div'> {
  isInteractive?: boolean;
}

/**
 * Animated app icon wrapper with hover effects
 */
export const MotionAppIconWrapper = forwardRef<HTMLDivElement, AppIconWrapperProps>(
  ({ isInteractive = true, children, ...props }, ref) => {
    return (
      <motion.div
        ref={ref}
        variants={VOICE_INPUT_VARIANTS.appIcon}
        initial="rest"
        whileHover={isInteractive ? "hover" : undefined}
        whileTap={isInteractive ? "tap" : undefined}
        {...getOptimizedMotionProps(['transform'])}
        {...props}
      >
        {children}
      </motion.div>
    );
  }
);

MotionAppIconWrapper.displayName = 'MotionAppIconWrapper';

// =============================================================================
// AUDIO LEVEL COMPONENTS
// =============================================================================

interface AudioLevelIndicatorProps extends HTMLMotionProps<'div'> {
  level: number; // 0-1 range
}

/**
 * Audio level reactive indicator
 */
export const MotionAudioLevelIndicator = forwardRef<HTMLDivElement, AudioLevelIndicatorProps>(
  ({ level, style, ...props }, ref) => {
    const normalizedLevel = Math.min(Math.max(level, 0), 1);
    
    // Determine animation state based on level
    const getLevelState = () => {
      if (normalizedLevel < 0.3) return 'low';
      if (normalizedLevel < 0.7) return 'medium';
      return 'high';
    };

    return (
      <motion.div
        ref={ref}
        variants={VOICE_INPUT_VARIANTS.audioLevel}
        animate={getLevelState()}
        style={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          width: '100%',
          height: '100%',
          borderRadius: 'inherit',
          pointerEvents: 'none',
          background: `radial-gradient(circle, rgba(99, 102, 241, ${normalizedLevel * 0.3}), transparent 70%)`,
          ...style,
        }}
        {...getOptimizedMotionProps(['transform', 'opacity', 'box-shadow'])}
        {...props}
      />
    );
  }
);

MotionAudioLevelIndicator.displayName = 'MotionAudioLevelIndicator';

// =============================================================================
// FINAL EXPORTS - Remove duplicate export block
// =============================================================================

// All components are already exported with their definitions above using named exports