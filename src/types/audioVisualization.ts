/**
 * Advanced Audio Visualization Types for Recording King
 * Provides TypeScript interfaces for FFT data, visualization configurations,
 * and Canvas-based rendering systems
 */

// =============================================================================
// AUDIO DATA INTERFACES
// =============================================================================

/**
 * Real-time FFT frequency spectrum data from Tauri backend
 */
export interface FFTData {
  /** Frequency bin values (0.0-1.0 normalized) */
  spectrum: Float32Array;
  /** Sample rate of the audio */
  sampleRate: number;
  /** Number of FFT bins */
  binCount: number;
  /** Timestamp of the data */
  timestamp: number;
  /** Peak frequency detected */
  peakFrequency?: number;
  /** Overall spectral centroid */
  spectralCentroid?: number;
}

/**
 * Audio level data with RMS and peak values
 */
export interface AudioLevelData {
  /** RMS audio level (0.0-1.0) */
  rms: number;
  /** Peak audio level (0.0-1.0) */
  peak: number;
  /** Timestamp of the measurement */
  timestamp: number;
  /** Whether audio input is active */
  isActive: boolean;
}

/**
 * Time-domain waveform data
 */
export interface WaveformData {
  /** Audio samples for waveform display */
  samples: Float32Array;
  /** Sample rate */
  sampleRate: number;
  /** Duration in seconds */
  duration: number;
  /** Timestamp */
  timestamp: number;
}

/**
 * Spectrogram data for time-frequency visualization
 */
export interface SpectrogramData {
  /** 2D array of frequency data over time [time][frequency] */
  timeFrequencyMatrix: Float32Array[];
  /** Time resolution in seconds per column */
  timeResolution: number;
  /** Frequency resolution in Hz per row */
  frequencyResolution: number;
  /** Maximum time span to keep in history */
  maxTimeSpan: number;
}

// =============================================================================
// VISUALIZATION CONFIGURATION
// =============================================================================

/**
 * Visualization mode types
 */
export type VisualizationMode = 'spectrum' | 'waveform' | 'spectrogram' | 'combined';

/**
 * Color scheme options for visualizations
 */
export type ColorScheme = 'purple' | 'blue' | 'green' | 'rainbow' | 'fire' | 'ice' | 'custom';

/**
 * Particle effect types for enhanced visualization
 */
export type ParticleEffect = 'none' | 'sparks' | 'fireflies' | 'waves' | 'burst';

/**
 * Base configuration for all audio visualizers
 */
export interface BaseVisualizationConfig {
  /** Visualization mode */
  mode: VisualizationMode;
  /** Color scheme */
  colorScheme: ColorScheme;
  /** Custom colors for 'custom' color scheme */
  customColors?: string[];
  /** Animation speed multiplier */
  animationSpeed: number;
  /** Sensitivity to audio changes */
  sensitivity: number;
  /** Whether to show frequency labels */
  showFrequencyLabels: boolean;
  /** Whether to enable particle effects */
  particleEffects: ParticleEffect;
  /** Performance mode settings */
  performance: {
    targetFPS: number;
    enableWebGL: boolean;
    enableOptimizations: boolean;
  };
}

/**
 * Spectrum visualizer specific configuration
 */
export interface SpectrumVisualizerConfig extends BaseVisualizationConfig {
  mode: 'spectrum';
  /** Number of frequency bars to display */
  barCount: number;
  /** Bar spacing factor */
  barSpacing: number;
  /** Logarithmic frequency scaling */
  logarithmicScale: boolean;
  /** Smoothing factor for bar animations */
  smoothing: number;
  /** Peak hold time in milliseconds */
  peakHoldTime: number;
  /** Whether to show peak dots */
  showPeaks: boolean;
  /** Frequency range to display */
  frequencyRange: {
    min: number;
    max: number;
  };
}

/**
 * Waveform visualizer specific configuration
 */
export interface WaveformVisualizerConfig extends BaseVisualizationConfig {
  mode: 'waveform';
  /** Time window to display in seconds */
  timeWindow: number;
  /** Line thickness */
  lineWidth: number;
  /** Whether to fill the waveform */
  filled: boolean;
  /** Waveform style */
  style: 'line' | 'bars' | 'curve' | 'mirror';
  /** Number of samples to display */
  sampleCount: number;
}

/**
 * Spectrogram visualizer specific configuration
 */
export interface SpectrogramVisualizerConfig extends BaseVisualizationConfig {
  mode: 'spectrogram';
  /** Time history to keep in seconds */
  timeHistory: number;
  /** Frequency resolution */
  frequencyBins: number;
  /** Color intensity mapping */
  intensityMapping: 'linear' | 'logarithmic' | 'power';
  /** Whether to scroll horizontally */
  scrolling: boolean;
  /** Heat map interpolation */
  interpolation: 'nearest' | 'bilinear' | 'smooth';
}

/**
 * Combined visualizer configuration
 */
export interface CombinedVisualizerConfig extends BaseVisualizationConfig {
  mode: 'combined';
  /** Which visualizations to show */
  enabledModes: ('spectrum' | 'waveform' | 'spectrogram')[];
  /** Layout arrangement */
  layout: 'horizontal' | 'vertical' | 'overlay' | 'grid';
  /** Relative sizes of each visualization */
  weights: Record<string, number>;
}

/**
 * Union type for all visualizer configurations
 */
export type VisualizerConfig = 
  | SpectrumVisualizerConfig 
  | WaveformVisualizerConfig 
  | SpectrogramVisualizerConfig 
  | CombinedVisualizerConfig;

// =============================================================================
// CANVAS RENDERING INTERFACES
// =============================================================================

/**
 * Canvas context with WebGL support detection
 */
export interface CanvasContext {
  /** 2D rendering context */
  ctx2d?: CanvasRenderingContext2D;
  /** WebGL rendering context */
  webgl?: WebGLRenderingContext;
  /** Canvas element */
  canvas: HTMLCanvasElement;
  /** Current rendering backend */
  backend: 'canvas2d' | 'webgl';
  /** Device pixel ratio for high-DPI displays */
  pixelRatio: number;
}

/**
 * Render frame data structure
 */
export interface RenderFrame {
  /** FFT data for this frame */
  fftData?: FFTData;
  /** Audio level data */
  audioLevel?: AudioLevelData;
  /** Waveform data */
  waveform?: WaveformData;
  /** Delta time since last frame in milliseconds */
  deltaTime: number;
  /** Frame timestamp */
  timestamp: number;
  /** Frame number */
  frameNumber: number;
}

/**
 * Rendering statistics
 */
export interface RenderStats {
  /** Current FPS */
  fps: number;
  /** Frame render time in milliseconds */
  frameTime: number;
  /** Memory usage in MB */
  memoryUsage: number;
  /** Number of drawn elements */
  drawCalls: number;
  /** Canvas backend being used */
  backend: 'canvas2d' | 'webgl';
}

// =============================================================================
// ANIMATION AND INTERACTION INTERFACES
// =============================================================================

/**
 * Animation state for smooth transitions
 */
export interface AnimationState {
  /** Current animation values */
  current: Record<string, number>;
  /** Target animation values */
  target: Record<string, number>;
  /** Animation velocity */
  velocity: Record<string, number>;
  /** Spring configuration */
  spring: {
    stiffness: number;
    damping: number;
    mass: number;
  };
}

/**
 * Particle system configuration
 */
export interface ParticleSystem {
  /** Maximum number of particles */
  maxParticles: number;
  /** Particle spawn rate per second */
  spawnRate: number;
  /** Particle lifetime in seconds */
  lifetime: number;
  /** Particle physics properties */
  physics: {
    gravity: number;
    friction: number;
    initialVelocity: { min: number; max: number };
  };
  /** Visual properties */
  visual: {
    size: { min: number; max: number };
    opacity: { start: number; end: number };
    color: string | string[];
  };
}

/**
 * Gesture interaction configuration
 */
export interface GestureConfig {
  /** Enable pan gestures */
  enablePan: boolean;
  /** Enable zoom gestures */
  enableZoom: boolean;
  /** Enable rotation gestures */
  enableRotation: boolean;
  /** Gesture sensitivity */
  sensitivity: number;
  /** Zoom limits */
  zoomLimits: {
    min: number;
    max: number;
  };
}

// =============================================================================
// EVENT INTERFACES
// =============================================================================

/**
 * Audio data events from Tauri backend
 */
export interface AudioDataEvent {
  /** Event type */
  type: 'audio_fft_data' | 'audio_level_update' | 'audio_waveform_data';
  /** Event payload */
  payload: FFTData | AudioLevelData | WaveformData;
  /** Event timestamp */
  timestamp: number;
}

/**
 * Visualization event handlers
 */
export interface VisualizationEventHandlers {
  /** Called when visualization mode changes */
  onModeChange?: (mode: VisualizationMode) => void;
  /** Called when configuration changes */
  onConfigChange?: (config: VisualizerConfig) => void;
  /** Called when rendering stats update */
  onStatsUpdate?: (stats: RenderStats) => void;
  /** Called on interaction events */
  onInteraction?: (event: InteractionEvent) => void;
  /** Called on errors */
  onError?: (error: VisualizationError) => void;
}

/**
 * Interaction event data
 */
export interface InteractionEvent {
  /** Interaction type */
  type: 'hover' | 'click' | 'drag' | 'zoom' | 'rotate';
  /** Mouse/touch position */
  position: { x: number; y: number };
  /** Additional event data */
  data?: any;
  /** Event timestamp */
  timestamp: number;
}

/**
 * Visualization error types
 */
export interface VisualizationError {
  /** Error code */
  code: 'WEBGL_NOT_SUPPORTED' | 'CANVAS_ERROR' | 'AUDIO_DATA_ERROR' | 'PERFORMANCE_ISSUE';
  /** Error message */
  message: string;
  /** Additional error context */
  context?: any;
  /** Error timestamp */
  timestamp: number;
}

// =============================================================================
// PERFORMANCE MONITORING
// =============================================================================

/**
 * Performance monitoring configuration
 */
export interface PerformanceMonitorConfig {
  /** Enable FPS monitoring */
  enableFpsMonitoring: boolean;
  /** Enable memory monitoring */
  enableMemoryMonitoring: boolean;
  /** Monitoring sample rate in Hz */
  sampleRate: number;
  /** Performance alert thresholds */
  thresholds: {
    minFps: number;
    maxFrameTime: number;
    maxMemoryUsage: number;
  };
}

/**
 * Performance metrics
 */
export interface PerformanceMetrics {
  /** FPS over time */
  fpsHistory: number[];
  /** Frame time history */
  frameTimeHistory: number[];
  /** Memory usage history */
  memoryHistory: number[];
  /** Current performance rating (0-1) */
  performanceRating: number;
  /** Recommendations for optimization */
  recommendations: string[];
}

// =============================================================================
// EXPORTS
// =============================================================================

export type {
  // Core data types
  FFTData,
  AudioLevelData,
  WaveformData,
  SpectrogramData,
  
  // Configuration types
  VisualizationMode,
  ColorScheme,
  ParticleEffect,
  VisualizerConfig,
  
  // Rendering types
  CanvasContext,
  RenderFrame,
  RenderStats,
  
  // Animation types
  AnimationState,
  ParticleSystem,
  GestureConfig,
  
  // Event types
  AudioDataEvent,
  VisualizationEventHandlers,
  InteractionEvent,
  VisualizationError,
  
  // Performance types
  PerformanceMonitorConfig,
  PerformanceMetrics,
};