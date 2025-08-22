/**
 * Motion Utilities for Recording King Voice Input System
 * Provides animation variants, accessibility handling, and performance optimization
 */

import type { Variants, Transition, MotionProps } from 'framer-motion';

// =============================================================================
// ACCESSIBILITY & PERFORMANCE
// =============================================================================

/**
 * Checks user's motion preferences and system capabilities
 */
export const getMotionPreferences = () => {
  const prefersReducedMotion = typeof window !== 'undefined' 
    ? window.matchMedia('(prefers-reduced-motion: reduce)').matches 
    : false;
    
  const supportsWebGL = typeof window !== 'undefined' 
    ? !!window.WebGLRenderingContext 
    : true;
    
  return {
    prefersReducedMotion,
    supportsWebGL,
    shouldAnimate: !prefersReducedMotion && supportsWebGL,
  };
};

/**
 * Creates reduced motion fallback variants
 */
export const createReducedMotionVariant = (variants: Variants): Variants => {
  const reduced: Variants = {};
  
  Object.keys(variants).forEach(key => {
    reduced[key] = {
      ...variants[key],
      transition: { duration: 0.01 },
      scale: variants[key]?.scale || 1,
      opacity: variants[key]?.opacity || 1,
      x: 0,
      y: 0,
    };
  });
  
  return reduced;
};

// =============================================================================
// CORE ANIMATION VARIANTS
// =============================================================================

/**
 * Voice Input Animation Variants System
 * Defines all animation states for voice input components
 */
export const VOICE_INPUT_VARIANTS = {
  // Container animations
  container: {
    hidden: {
      opacity: 0,
      scale: 0.9,
      y: 30,
      filter: 'blur(10px)',
    },
    visible: {
      opacity: 1,
      scale: 1,
      y: 0,
      filter: 'blur(0px)',
      transition: {
        type: 'spring',
        stiffness: 400,
        damping: 25,
        mass: 0.8,
        when: 'beforeChildren',
        staggerChildren: 0.1,
      },
    },
    exit: {
      opacity: 0,
      scale: 0.95,
      y: 10,
      filter: 'blur(5px)',
      transition: {
        duration: 0.2,
        ease: 'easeInOut',
      },
    },
  },

  // State-based container variants
  idle: {
    scale: 1,
    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.2)',
    transition: { duration: 0.3 },
  },
  
  listening: {
    scale: 1.02,
    boxShadow: [
      '0 8px 32px rgba(0, 0, 0, 0.2)',
      '0 0 20px rgba(99, 102, 241, 0.3)',
      '0 0 40px rgba(99, 102, 241, 0.1)',
    ].join(', '),
    transition: {
      scale: { duration: 0.3 },
      boxShadow: { duration: 0.5, repeat: Infinity, repeatType: 'reverse' as const },
    },
  },
  
  processing: {
    scale: 1,
    filter: 'brightness(1.1)',
    transition: {
      filter: { duration: 1.5, repeat: Infinity, repeatType: 'reverse' as const },
    },
  },
  
  injecting: {
    scale: 1.05,
    boxShadow: [
      '0 8px 32px rgba(0, 0, 0, 0.2)',
      '0 0 40px rgba(16, 185, 129, 0.6)',
      '0 0 80px rgba(16, 185, 129, 0.3)',
    ].join(', '),
    transition: { duration: 0.8, ease: 'easeOut' },
  },

  // Button animations
  button: {
    rest: {
      scale: 1,
      boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)',
    },
    hover: {
      scale: 1.05,
      boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
      transition: { duration: 0.2 },
    },
    tap: {
      scale: 0.95,
      transition: { duration: 0.1 },
    },
  },

  // Close button with rotation
  closeButton: {
    rest: {
      scale: 1,
      rotate: 0,
      backgroundColor: 'rgba(255, 255, 255, 0.08)',
    },
    hover: {
      scale: 1.2,
      rotate: 90,
      backgroundColor: 'rgba(239, 68, 68, 0.15)',
      transition: { duration: 0.3, ease: 'anticipate' },
    },
    tap: {
      scale: 1.1,
      rotate: 90,
    },
  },

  // Waveform bars
  waveformBar: {
    idle: {
      scaleY: 1,
      opacity: 0.7,
    },
    active: {
      scaleY: [1, 1.8, 1],
      opacity: [0.7, 1, 0.7],
      transition: {
        duration: 0.6,
        repeat: Infinity,
        ease: 'easeInOut',
      },
    },
  },

  // Text animations
  text: {
    hidden: {
      opacity: 0,
      y: 5,
      scale: 0.9,
    },
    visible: {
      opacity: 1,
      y: 0,
      scale: 1,
      transition: {
        duration: 0.3,
        ease: 'easeOut',
      },
    },
    typing: {
      opacity: 1,
      transition: {
        opacity: { duration: 0.1 },
      },
    },
  },

  // App icon animations
  appIcon: {
    rest: {
      scale: 1,
      rotate: 0,
    },
    hover: {
      scale: 1.1,
      rotate: 5,
      transition: { duration: 0.2 },
    },
    tap: {
      scale: 0.95,
      rotate: 0,
    },
  },

  // Processing spinner
  spinner: {
    spin: {
      rotate: 360,
      transition: {
        duration: 0.8,
        repeat: Infinity,
        ease: 'linear',
      },
    },
  },

  // Success checkmark
  successCheck: {
    hidden: {
      pathLength: 0,
      opacity: 0,
    },
    visible: {
      pathLength: 1,
      opacity: 1,
      transition: {
        pathLength: { duration: 0.5, ease: 'easeInOut' },
        opacity: { duration: 0.2 },
      },
    },
  },

  // Audio level indicator
  audioLevel: {
    low: {
      scale: 1,
      opacity: 0.5,
    },
    medium: {
      scale: 1.1,
      opacity: 0.7,
      transition: { duration: 0.2 },
    },
    high: {
      scale: 1.2,
      opacity: 1,
      boxShadow: '0 0 20px rgba(99, 102, 241, 0.5)',
      transition: { duration: 0.15 },
    },
  },
} as const;

// =============================================================================
// SPRING CONFIGURATIONS
// =============================================================================

export const SPRING_CONFIGS = {
  gentle: {
    type: 'spring' as const,
    stiffness: 200,
    damping: 20,
    mass: 1,
  },
  
  bouncy: {
    type: 'spring' as const,
    stiffness: 400,
    damping: 25,
    mass: 0.8,
  },
  
  snappy: {
    type: 'spring' as const,
    stiffness: 500,
    damping: 30,
    mass: 0.6,
  },
  
  wobbly: {
    type: 'spring' as const,
    stiffness: 300,
    damping: 15,
    mass: 1.2,
  },
} as const;

// =============================================================================
// ANIMATION UTILITY FUNCTIONS
// =============================================================================

/**
 * Creates a responsive transition based on motion preferences
 */
export const createResponsiveTransition = (
  transition: Transition,
  options?: { skipReduced?: boolean }
): Transition => {
  const { prefersReducedMotion } = getMotionPreferences();
  
  if (prefersReducedMotion && !options?.skipReduced) {
    return { duration: 0.01 };
  }
  
  return transition;
};

/**
 * Creates audio-reactive animation values based on audio level
 */
export const createAudioReactiveValues = (audioLevel: number) => {
  const normalizedLevel = Math.min(Math.max(audioLevel, 0), 1);
  
  return {
    scale: 1 + normalizedLevel * 0.2,
    opacity: 0.5 + normalizedLevel * 0.5,
    glowIntensity: normalizedLevel,
    waveformHeight: 8 + normalizedLevel * 12,
  };
};

/**
 * Creates staggered animation for multiple elements
 */
export const createStaggeredAnimation = (
  childCount: number,
  baseDelay: number = 0.1
): Transition => ({
  when: 'beforeChildren' as const,
  staggerChildren: baseDelay,
  delayChildren: baseDelay * 0.5,
});

/**
 * Performance-optimized motion props with will-change hints
 */
export const getOptimizedMotionProps = (
  willChangeProperties: string[] = ['transform', 'opacity']
): Partial<MotionProps> => ({
  style: {
    willChange: willChangeProperties.join(', '),
  },
  layoutDependency: false, // Disable layout animations for performance
});

/**
 * Creates gesture-based animation variants
 */
export const createGestureVariants = () => ({
  rest: { scale: 1 },
  hover: { scale: 1.05 },
  tap: { scale: 0.95 },
});

// =============================================================================
// AUDIO-REACTIVE ANIMATION CLASS
// =============================================================================

export class AudioReactiveAnimator {
  private audioLevel: number = 0;
  private isActive: boolean = false;
  private callbacks: Set<(level: number) => void> = new Set();
  
  constructor() {
    this.bindMethods();
  }
  
  private bindMethods() {
    this.updateAudioLevel = this.updateAudioLevel.bind(this);
    this.subscribe = this.subscribe.bind(this);
    this.unsubscribe = this.unsubscribe.bind(this);
  }
  
  updateAudioLevel(level: number): void {
    this.audioLevel = Math.min(Math.max(level, 0), 1);
    
    if (this.isActive) {
      this.callbacks.forEach(callback => callback(this.audioLevel));
    }
  }
  
  subscribe(callback: (level: number) => void): () => void {
    this.callbacks.add(callback);
    return () => this.unsubscribe(callback);
  }
  
  unsubscribe(callback: (level: number) => void): void {
    this.callbacks.delete(callback);
  }
  
  start(): void {
    this.isActive = true;
  }
  
  stop(): void {
    this.isActive = false;
    this.audioLevel = 0;
    this.callbacks.forEach(callback => callback(0));
  }
  
  getCurrentLevel(): number {
    return this.audioLevel;
  }
  
  getReactiveValues() {
    return createAudioReactiveValues(this.audioLevel);
  }
}

// =============================================================================
// EXPORTS
// =============================================================================

export type VoiceInputVariants = typeof VOICE_INPUT_VARIANTS;
export type SpringConfig = typeof SPRING_CONFIGS[keyof typeof SPRING_CONFIGS];

// Create a global instance for the audio reactive animator
export const audioReactiveAnimator = new AudioReactiveAnimator();