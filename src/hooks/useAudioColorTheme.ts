/**
 * Dynamic Audio Color Theme Hook
 * Generates color schemes based on real-time audio characteristics
 * for Recording King voice input system
 */

import { useState, useEffect, useRef, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';

interface AudioColorTheme {
  primary: string;
  secondary: string;
  accent: string;
  background: string;
  gradient: string;
  waveformColor: string;
  pulseColor: string;
  textColor: string;
}

interface FFTData {
  spectrum: number[];
  peak_frequency?: number;
  spectral_centroid?: number;
  timestamp: number;
}

interface AudioCharacteristics {
  dominantFrequency: number;
  spectralCentroid: number;
  energyLevel: number;
  bassLevel: number;
  midLevel: number;
  trebleLevel: number;
}

const DEFAULT_THEME: AudioColorTheme = {
  primary: 'rgba(99, 102, 241, 0.9)',
  secondary: 'rgba(168, 85, 247, 0.9)', 
  accent: 'rgba(59, 130, 246, 0.8)',
  background: 'rgba(15, 23, 42, 0.95)',
  gradient: 'linear-gradient(135deg, rgba(99, 102, 241, 0.9), rgba(168, 85, 247, 0.9))',
  waveformColor: 'rgba(34, 197, 94, 0.8)',
  pulseColor: 'rgba(236, 72, 153, 0.6)',
  textColor: 'rgba(248, 250, 252, 0.95)',
};

/**
 * Hook for generating dynamic color themes based on audio characteristics
 */
export const useAudioColorTheme = (
  isActive: boolean = false,
  sensitivity: number = 1.0
): {
  theme: AudioColorTheme;
  characteristics: AudioCharacteristics | null;
  updateTheme: (audioData: FFTData) => void;
  resetTheme: () => void;
} => {
  const [theme, setTheme] = useState<AudioColorTheme>(DEFAULT_THEME);
  const [characteristics, setCharacteristics] = useState<AudioCharacteristics | null>(null);
  const themeUpdateTimeoutRef = useRef<NodeJS.Timeout>();
  const lastUpdateRef = useRef<number>(0);

  // Analyze audio characteristics from FFT data
  const analyzeAudioCharacteristics = useCallback((fftData: FFTData): AudioCharacteristics => {
    const spectrum = fftData.spectrum;
    const totalBins = spectrum.length;
    
    // Frequency band analysis (assuming 16kHz sample rate, 1024 FFT)
    const bassEnd = Math.floor(totalBins * 0.1);    // 0-800 Hz
    const midEnd = Math.floor(totalBins * 0.4);     // 800-3200 Hz
    const trebleEnd = totalBins;                     // 3200-8000 Hz
    
    const bassLevel = spectrum.slice(0, bassEnd).reduce((sum, val) => sum + val, 0) / bassEnd;
    const midLevel = spectrum.slice(bassEnd, midEnd).reduce((sum, val) => sum + val, 0) / (midEnd - bassEnd);
    const trebleLevel = spectrum.slice(midEnd, trebleEnd).reduce((sum, val) => sum + val, 0) / (trebleEnd - midEnd);
    
    const energyLevel = spectrum.reduce((sum, val) => sum + val * val, 0) / totalBins;
    
    return {
      dominantFrequency: fftData.peak_frequency || 0,
      spectralCentroid: fftData.spectral_centroid || 0,
      energyLevel: Math.min(energyLevel * sensitivity, 1.0),
      bassLevel: Math.min(bassLevel * sensitivity, 1.0),
      midLevel: Math.min(midLevel * sensitivity, 1.0),
      trebleLevel: Math.min(trebleLevel * sensitivity, 1.0),
    };
  }, [sensitivity]);

  // Generate color theme from audio characteristics
  const generateThemeFromAudio = useCallback((chars: AudioCharacteristics): AudioColorTheme => {
    // Base hue calculation from dominant frequency (100-4000 Hz -> 240-300 degrees)
    const frequencyHue = Math.min(240 + (chars.dominantFrequency / 4000) * 60, 300);
    
    // Saturation based on energy level (60-90%)
    const saturation = Math.floor(60 + chars.energyLevel * 30);
    
    // Lightness based on spectral centroid (40-70%)
    const lightness = Math.floor(40 + (chars.spectralCentroid / 4000) * 30);
    
    // Bass-driven warm colors, treble-driven cool colors
    const warmCoolBalance = chars.bassLevel - chars.trebleLevel;
    const hueShift = warmCoolBalance * 30; // ±30 degrees
    const primaryHue = Math.max(0, Math.min(360, frequencyHue + hueShift));
    
    // Secondary hue with complementary relationship
    const secondaryHue = (primaryHue + 120) % 360;
    
    // Accent hue with triadic relationship
    const accentHue = (primaryHue + 240) % 360;
    
    // Energy-based alpha values
    const primaryAlpha = 0.7 + chars.energyLevel * 0.2;
    const secondaryAlpha = 0.6 + chars.energyLevel * 0.3;
    const accentAlpha = 0.5 + chars.energyLevel * 0.3;
    
    return {
      primary: `hsla(${primaryHue}, ${saturation}%, ${lightness}%, ${primaryAlpha})`,
      secondary: `hsla(${secondaryHue}, ${saturation}%, ${lightness + 10}%, ${secondaryAlpha})`,
      accent: `hsla(${accentHue}, ${saturation + 10}%, ${lightness + 5}%, ${accentAlpha})`,
      background: `hsla(${primaryHue}, ${Math.floor(saturation * 0.3)}%, ${Math.floor(lightness * 0.3)}%, 0.95)`,
      gradient: `linear-gradient(135deg, 
        hsla(${primaryHue}, ${saturation}%, ${lightness}%, ${primaryAlpha}), 
        hsla(${secondaryHue}, ${saturation}%, ${lightness + 10}%, ${secondaryAlpha}))`,
      waveformColor: `hsla(${primaryHue + 60}, ${saturation}%, ${lightness + 20}%, ${0.6 + chars.midLevel * 0.4})`,
      pulseColor: `hsla(${accentHue}, ${saturation + 20}%, ${lightness + 15}%, ${0.4 + chars.energyLevel * 0.4})`,
      textColor: `hsla(${primaryHue}, ${Math.floor(saturation * 0.2)}%, ${85 + chars.energyLevel * 10}%, 0.95)`,
    };
  }, []);

  // Update theme with throttling for performance
  const updateTheme = useCallback((audioData: FFTData) => {
    const now = performance.now();
    if (now - lastUpdateRef.current < 50) return; // Throttle to 20fps
    
    lastUpdateRef.current = now;
    
    const audioChars = analyzeAudioCharacteristics(audioData);
    setCharacteristics(audioChars);
    
    if (isActive) {
      const newTheme = generateThemeFromAudio(audioChars);
      setTheme(newTheme);
      
      // Auto-reset to default after silence
      if (themeUpdateTimeoutRef.current) {
        clearTimeout(themeUpdateTimeoutRef.current);
      }
      
      themeUpdateTimeoutRef.current = setTimeout(() => {
        if (audioChars.energyLevel < 0.1) {
          setTheme(DEFAULT_THEME);
        }
      }, 2000);
    }
  }, [isActive, analyzeAudioCharacteristics, generateThemeFromAudio]);

  // Reset theme to default
  const resetTheme = useCallback(() => {
    setTheme(DEFAULT_THEME);
    setCharacteristics(null);
    if (themeUpdateTimeoutRef.current) {
      clearTimeout(themeUpdateTimeoutRef.current);
    }
  }, []);

  // Listen to FFT data events from Tauri backend
  useEffect(() => {
    if (!isActive) {
      resetTheme();
      return;
    }

    const setupEventListener = async () => {
      const unlisten = await listen<FFTData>('audio_fft_data', (event) => {
        updateTheme(event.payload);
      });
      
      return unlisten;
    };

    setupEventListener();
  }, [isActive, updateTheme, resetTheme]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (themeUpdateTimeoutRef.current) {
        clearTimeout(themeUpdateTimeoutRef.current);
      }
    };
  }, []);

  return {
    theme,
    characteristics,
    updateTheme,
    resetTheme,
  };
};

/**
 * Utility functions for color theme manipulation
 */
export const audioColorUtils = {
  // Apply theme to CSS custom properties
  applyThemeToCSS: (theme: AudioColorTheme, prefix: string = '--audio') => {
    document.documentElement.style.setProperty(`${prefix}-primary`, theme.primary);
    document.documentElement.style.setProperty(`${prefix}-secondary`, theme.secondary);
    document.documentElement.style.setProperty(`${prefix}-accent`, theme.accent);
    document.documentElement.style.setProperty(`${prefix}-background`, theme.background);
    document.documentElement.style.setProperty(`${prefix}-gradient`, theme.gradient);
    document.documentElement.style.setProperty(`${prefix}-waveform`, theme.waveformColor);
    document.documentElement.style.setProperty(`${prefix}-pulse`, theme.pulseColor);
    document.documentElement.style.setProperty(`${prefix}-text`, theme.textColor);
  },

  // Generate contrasting text color
  getContrastingTextColor: (backgroundColor: string): string => {
    // Simple contrast calculation
    const rgb = backgroundColor.match(/\d+/g);
    if (!rgb || rgb.length < 3) return '#ffffff';
    
    const brightness = (parseInt(rgb[0]) * 299 + parseInt(rgb[1]) * 587 + parseInt(rgb[2]) * 114) / 1000;
    return brightness > 128 ? '#000000' : '#ffffff';
  },

  // Create gradient from theme colors
  createAudioGradient: (theme: AudioColorTheme, direction: string = '135deg'): string => {
    return `linear-gradient(${direction}, ${theme.primary}, ${theme.secondary}, ${theme.accent})`;
  },
};

export default useAudioColorTheme;