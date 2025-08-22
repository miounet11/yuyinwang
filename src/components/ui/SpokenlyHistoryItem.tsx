/**
 * SpokenlyHistoryItem - History record display component
 * Provides history item with timestamp, content preview, and actions
 */

import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { HistoryItemProps } from './types';
import { SpokenlyButton } from './SpokenlyButton';

export const SpokenlyHistoryItem: React.FC<HistoryItemProps> = ({
  id,
  title,
  content,
  timestamp,
  duration,
  fileSize,
  format,
  isSelected = false,
  onSelect,
  onPlay,
  onDelete,
  onExport,
  className = ''
}) => {
  const [isExpanded, setIsExpanded] = useState(false);

  // Format duration
  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  };

  // Format file size
  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  };

  // Format timestamp
  const formatTimestamp = (date: Date) => {
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    
    if (days === 0) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (days === 1) {
      return 'Yesterday';
    } else if (days < 7) {
      return `${days} days ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  const handleItemClick = () => {
    if (onSelect) {
      onSelect(id);
    }
  };

  const handleToggleExpand = (e: React.MouseEvent) => {
    e.stopPropagation();
    setIsExpanded(!isExpanded);
  };

  return (
    <motion.div
      className={`spokenly-history-item ${className} ${isSelected ? 'selected' : ''}`}
      style={{
        backgroundColor: isSelected ? 'var(--spokenly-bg-selected)' : 'var(--spokenly-bg-content)',
        border: `1px solid ${isSelected ? 'var(--spokenly-border-selected)' : 'var(--spokenly-border-primary)'}`,
        borderRadius: 'var(--spokenly-radius-lg)',
        padding: 'var(--spokenly-space-4)',
        cursor: onSelect ? 'pointer' : 'default',
        transition: 'all var(--spokenly-duration-fast) var(--spokenly-ease-out)',
        position: 'relative',
        marginBottom: 'var(--spokenly-space-3)'
      }}
      whileHover={onSelect ? {
        boxShadow: 'var(--spokenly-shadow-hover)',
        borderColor: 'var(--spokenly-border-secondary)'
      } : {}}
      onClick={handleItemClick}
      layout
    >
      {/* Header */}
      <div
        className="spokenly-history-header"
        style={{
          display: 'flex',
          alignItems: 'flex-start',
          justifyContent: 'space-between',
          marginBottom: 'var(--spokenly-space-3)'
        }}
      >
        <div className="spokenly-history-meta" style={{ flex: 1 }}>
          {/* Title */}
          <h3
            className="spokenly-history-title"
            style={{
              fontSize: 'var(--spokenly-text-base)',
              fontWeight: 'var(--spokenly-font-semibold)',
              color: 'var(--spokenly-text-primary)',
              margin: '0 0 4px 0',
              lineHeight: 'var(--spokenly-leading-tight)'
            }}
          >
            {title}
          </h3>

          {/* Metadata */}
          <div
            className="spokenly-history-info"
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 'var(--spokenly-space-3)',
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)'
            }}
          >
            <span>{formatTimestamp(timestamp)}</span>
            {duration && (
              <>
                <span>•</span>
                <span>{formatDuration(duration)}</span>
              </>
            )}
            {fileSize && (
              <>
                <span>•</span>
                <span>{formatFileSize(fileSize)}</span>
              </>
            )}
            {format && (
              <>
                <span>•</span>
                <span style={{ 
                  textTransform: 'uppercase', 
                  fontWeight: 'var(--spokenly-font-medium)',
                  color: 'var(--spokenly-primary)'
                }}>
                  {format}
                </span>
              </>
            )}
          </div>
        </div>

        {/* Actions */}
        <div
          className="spokenly-history-actions"
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: 'var(--spokenly-space-2)',
            marginLeft: 'var(--spokenly-space-4)'
          }}
        >
          {onPlay && (
            <SpokenlyButton
              size="sm"
              variant="ghost"
              onClick={(e) => {
                e.stopPropagation();
                onPlay(id);
              }}
              leftIcon={
                <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
                  <path d="M3.5 2v10l7-5-7-5z" />
                </svg>
              }
            >
              Play
            </SpokenlyButton>
          )}

          {onExport && (
            <SpokenlyButton
              size="sm"
              variant="ghost"
              onClick={(e) => {
                e.stopPropagation();
                onExport(id);
              }}
              leftIcon={
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor">
                  <path d="M7 1v8m0 0l3-3m-3 3L4 6" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
                  <path d="M1 10v2a1 1 0 0 0 1 1h10a1 1 0 0 0 1-1v-2" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              }
            >
              Export
            </SpokenlyButton>
          )}

          {onDelete && (
            <SpokenlyButton
              size="sm"
              variant="ghost"
              onClick={(e) => {
                e.stopPropagation();
                onDelete(id);
              }}
              leftIcon={
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor">
                  <path d="M1 3h12m-10 0v8a1 1 0 0 0 1 1h6a1 1 0 0 0 1-1V3M4.5 3V2a1 1 0 0 1 1-1h3a1 1 0 0 1 1 1v1" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
                </svg>
              }
              style={{ color: 'var(--spokenly-error)' }}
            >
              Delete
            </SpokenlyButton>
          )}
        </div>
      </div>

      {/* Content Preview */}
      {content && (
        <div className="spokenly-history-content">
          <motion.div
            className="spokenly-history-preview"
            style={{
              fontSize: 'var(--spokenly-text-sm)',
              color: 'var(--spokenly-text-secondary)',
              lineHeight: 'var(--spokenly-leading-relaxed)',
              backgroundColor: 'var(--spokenly-bg-hover)',
              borderRadius: 'var(--spokenly-radius-base)',
              padding: 'var(--spokenly-space-3)',
              overflow: 'hidden',
              position: 'relative'
            }}
            animate={{ 
              maxHeight: isExpanded ? 'none' : '60px'
            }}
            transition={{ duration: 0.3, ease: [0.0, 0.0, 0.2, 1] }}
          >
            {content}
            
            {/* Gradient Fade */}
            {!isExpanded && content.length > 100 && (
              <div
                style={{
                  position: 'absolute',
                  bottom: 0,
                  left: 0,
                  right: 0,
                  height: '20px',
                  background: 'linear-gradient(transparent, var(--spokenly-bg-hover))',
                  pointerEvents: 'none'
                }}
              />
            )}
          </motion.div>

          {/* Expand/Collapse Button */}
          {content.length > 100 && (
            <motion.button
              className="spokenly-history-expand"
              onClick={handleToggleExpand}
              style={{
                fontSize: 'var(--spokenly-text-sm)',
                color: 'var(--spokenly-primary)',
                background: 'none',
                border: 'none',
                cursor: 'pointer',
                marginTop: 'var(--spokenly-space-2)',
                padding: 0,
                fontWeight: 'var(--spokenly-font-medium)',
                display: 'flex',
                alignItems: 'center',
                gap: 'var(--spokenly-space-1)'
              }}
              whileHover={{ color: 'var(--spokenly-primary-hover)' }}
            >
              {isExpanded ? 'Show less' : 'Show more'}
              <motion.svg
                width="12"
                height="12"
                viewBox="0 0 12 12"
                fill="currentColor"
                animate={{ rotate: isExpanded ? 180 : 0 }}
                transition={{ duration: 0.2 }}
              >
                <path d="M2 4l4 4 4-4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" fill="none"/>
              </motion.svg>
            </motion.button>
          )}
        </div>
      )}

      {/* Selection Indicator */}
      {isSelected && (
        <motion.div
          className="spokenly-history-indicator"
          layoutId="selectedHistory"
          style={{
            position: 'absolute',
            left: 0,
            top: 0,
            bottom: 0,
            width: '4px',
            backgroundColor: 'var(--spokenly-primary)',
            borderRadius: '0 2px 2px 0'
          }}
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
        />
      )}
    </motion.div>
  );
};