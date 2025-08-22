/**
 * Progressive Voice Input Settings with Motion Enhancement
 * Enhanced version of the existing settings component with Framer Motion
 */

import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  MotionWrapper,
  MotionButton,
  MotionText,
} from './MotionComponents';
import { VOICE_INPUT_VARIANTS, SPRING_CONFIGS } from '../../utils/motionUtils';

interface MotionProgressiveVoiceInputSettingsProps {
  isVisible: boolean;
  onClose: () => void;
  currentModel: string;
  onModelChange: (model: string) => void;
}

export const MotionProgressiveVoiceInputSettings: React.FC<MotionProgressiveVoiceInputSettingsProps> = ({
  isVisible,
  onClose,
  currentModel,
  onModelChange,
}) => {
  const [selectedModel, setSelectedModel] = useState(currentModel);
  const [isChangingModel, setIsChangingModel] = useState(false);

  const handleModelChange = async (model: string) => {
    setIsChangingModel(true);
    setSelectedModel(model);
    
    // Simulate model change delay
    setTimeout(() => {
      onModelChange(model);
      setIsChangingModel(false);
    }, 1000);
  };

  const containerVariants = {
    hidden: {
      opacity: 0,
      scale: 0.9,
      y: 20,
    },
    visible: {
      opacity: 1,
      scale: 1,
      y: 0,
      transition: {
        ...SPRING_CONFIGS.bouncy,
        staggerChildren: 0.1,
      },
    },
    exit: {
      opacity: 0,
      scale: 0.95,
      y: 10,
      transition: { duration: 0.2 },
    },
  };

  const itemVariants = {
    hidden: { opacity: 0, x: -20 },
    visible: { 
      opacity: 1, 
      x: 0,
      transition: SPRING_CONFIGS.gentle,
    },
  };

  const models = [
    { id: 'whisper-1', name: 'Whisper v1 (Fast)', description: 'Quick transcription' },
    { id: 'whisper-large', name: 'Whisper Large (Accurate)', description: 'High accuracy' },
    { id: 'deepgram-nova', name: 'Deepgram Nova', description: 'Real-time streaming' },
  ];

  return (
    <AnimatePresence>
      {isVisible && (
        <motion.div
          className="progressive-voice-settings-overlay"
          variants={containerVariants}
          initial="hidden"
          animate="visible"
          exit="exit"
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0, 0, 0, 0.8)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
          onClick={onClose}
        >
          <MotionWrapper
            className="settings-panel"
            onClick={(e) => e.stopPropagation()}
            style={{
              background: 'rgba(30, 30, 30, 0.95)',
              backdropFilter: 'blur(20px)',
              borderRadius: '16px',
              padding: '24px',
              minWidth: '320px',
              border: '1px solid rgba(255, 255, 255, 0.1)',
            }}
            variants={itemVariants}
          >
            {/* Header */}
            <div style={{ marginBottom: '20px' }}>
              <MotionText
                style={{
                  fontSize: '18px',
                  fontWeight: '600',
                  color: 'rgba(255, 255, 255, 0.9)',
                  marginBottom: '8px',
                }}
                variants={itemVariants}
              >
                Voice Input Settings
              </MotionText>
              <MotionText
                style={{
                  fontSize: '14px',
                  color: 'rgba(255, 255, 255, 0.6)',
                }}
                variants={itemVariants}
              >
                Configure your transcription model
              </MotionText>
            </div>

            {/* Model Selection */}
            <MotionWrapper variants={itemVariants}>
              <MotionText
                style={{
                  fontSize: '14px',
                  fontWeight: '500',
                  color: 'rgba(255, 255, 255, 0.8)',
                  marginBottom: '12px',
                }}
              >
                Transcription Model
              </MotionText>

              <div style={{ marginBottom: '20px' }}>
                {models.map((model, index) => (
                  <motion.div
                    key={model.id}
                    variants={itemVariants}
                    custom={index}
                    whileHover={{ scale: 1.02 }}
                    whileTap={{ scale: 0.98 }}
                    style={{
                      padding: '12px',
                      margin: '8px 0',
                      borderRadius: '8px',
                      border: '1px solid rgba(255, 255, 255, 0.1)',
                      background: selectedModel === model.id 
                        ? 'rgba(99, 102, 241, 0.2)' 
                        : 'rgba(255, 255, 255, 0.05)',
                      cursor: 'pointer',
                      transition: 'all 0.2s ease',
                    }}
                    onClick={() => handleModelChange(model.id)}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                      <div>
                        <div style={{ 
                          color: 'rgba(255, 255, 255, 0.9)', 
                          fontWeight: '500', 
                          marginBottom: '4px' 
                        }}>
                          {model.name}
                          {isChangingModel && selectedModel === model.id && (
                            <motion.span
                              animate={{ rotate: 360 }}
                              transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
                              style={{ marginLeft: '8px', display: 'inline-block' }}
                            >
                              ⟳
                            </motion.span>
                          )}
                        </div>
                        <div style={{ 
                          color: 'rgba(255, 255, 255, 0.6)', 
                          fontSize: '12px' 
                        }}>
                          {model.description}
                        </div>
                      </div>
                      {selectedModel === model.id && (
                        <motion.div
                          initial={{ scale: 0 }}
                          animate={{ scale: 1 }}
                          style={{
                            width: '16px',
                            height: '16px',
                            borderRadius: '50%',
                            background: 'rgba(99, 102, 241, 1)',
                            display: 'flex',
                            alignItems: 'center',
                            justifyContent: 'center',
                            color: 'white',
                            fontSize: '10px',
                          }}
                        >
                          ✓
                        </motion.div>
                      )}
                    </div>
                  </motion.div>
                ))}
              </div>
            </MotionWrapper>

            {/* Action Buttons */}
            <MotionWrapper 
              style={{ 
                display: 'flex', 
                gap: '12px', 
                justifyContent: 'flex-end' 
              }}
              variants={itemVariants}
            >
              <MotionButton
                onClick={onClose}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  border: '1px solid rgba(255, 255, 255, 0.2)',
                  background: 'rgba(255, 255, 255, 0.05)',
                  color: 'rgba(255, 255, 255, 0.8)',
                  fontSize: '14px',
                }}
              >
                Cancel
              </MotionButton>
              
              <MotionButton
                onClick={onClose}
                isProcessing={isChangingModel}
                style={{
                  padding: '8px 16px',
                  borderRadius: '6px',
                  background: 'linear-gradient(135deg, #6366f1, #8b5cf6)',
                  border: 'none',
                  color: 'white',
                  fontSize: '14px',
                }}
              >
                {isChangingModel ? 'Applying...' : 'Done'}
              </MotionButton>
            </MotionWrapper>
          </MotionWrapper>
        </motion.div>
      )}
    </AnimatePresence>
  );
};

export default MotionProgressiveVoiceInputSettings;