/**
 * Spokenly Design System - TypeScript Interfaces
 * Core type definitions for all UI components
 */

import { ReactNode, HTMLAttributes, ButtonHTMLAttributes, InputHTMLAttributes } from 'react';

// Base component props
export interface BaseComponentProps {
  className?: string;
  children?: ReactNode;
}

// Size variants
export type Size = 'sm' | 'md' | 'lg';
export type ButtonSize = 'xs' | 'sm' | 'md' | 'lg';

// Color variants
export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger' | 'success';
export type TagVariant = 'default' | 'success' | 'warning' | 'error' | 'info';

// Layout Props
export interface LayoutProps extends BaseComponentProps {
  children: ReactNode;
}

export interface SidebarProps extends BaseComponentProps {
  isCollapsed?: boolean;
  onToggle?: () => void;
  width?: number;
}

export interface ContentProps extends BaseComponentProps {
  sidebarWidth?: number;
  padding?: Size;
}

// Navigation Props
export interface NavItemProps extends BaseComponentProps {
  icon?: ReactNode;
  label: string;
  isActive?: boolean;
  isDisabled?: boolean;
  onClick?: () => void;
  href?: string;
  badge?: string | number;
}

export interface NavSectionProps extends BaseComponentProps {
  title?: string;
  children: ReactNode;
}

// Button Props
export interface ButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'size'>, BaseComponentProps {
  variant?: ButtonVariant;
  size?: ButtonSize;
  isLoading?: boolean;
  loadingText?: string;
  leftIcon?: ReactNode;
  rightIcon?: ReactNode;
  fullWidth?: boolean;
}

// Card Props
export interface CardProps extends HTMLAttributes<HTMLDivElement>, BaseComponentProps {
  padding?: Size;
  hover?: boolean;
  selected?: boolean;
}

export interface CardHeaderProps extends BaseComponentProps {
  title?: string;
  subtitle?: string;
  actions?: ReactNode;
}

export interface CardBodyProps extends BaseComponentProps {
  padding?: Size;
}

export interface CardFooterProps extends BaseComponentProps {
  justify?: 'start' | 'center' | 'end' | 'between';
}

// Input Props
export interface InputProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'>, BaseComponentProps {
  label?: string;
  helperText?: string;
  errorText?: string;
  leftIcon?: ReactNode;
  rightIcon?: ReactNode;
  size?: Size;
  fullWidth?: boolean;
}

// Switch Props
export interface SwitchProps extends BaseComponentProps {
  checked?: boolean;
  defaultChecked?: boolean;
  onCheckedChange?: (checked: boolean) => void;
  disabled?: boolean;
  label?: string;
  description?: string;
  size?: Size;
}

// Select Props
export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
  icon?: ReactNode;
}

export interface SelectProps extends BaseComponentProps {
  value?: string;
  defaultValue?: string;
  onValueChange?: (value: string) => void;
  placeholder?: string;
  options: SelectOption[];
  disabled?: boolean;
  label?: string;
  errorText?: string;
  size?: Size;
  fullWidth?: boolean;
}

// Tag Props
export interface TagProps extends BaseComponentProps {
  variant?: TagVariant;
  size?: Size;
  removable?: boolean;
  onRemove?: () => void;
  icon?: ReactNode;
}

// Model Card Props
export interface ModelStatus {
  type: 'online' | 'offline' | 'loading' | 'error';
  message?: string;
}

export interface ModelCardProps extends BaseComponentProps {
  title: string;
  description?: string;
  provider?: string;
  status: ModelStatus;
  isSelected?: boolean;
  onSelect?: () => void;
  tags?: string[];
  actions?: ReactNode;
  image?: string;
  pricing?: string;
}

// Upload Area Props
export interface UploadAreaProps extends BaseComponentProps {
  onFilesDrop?: (files: FileList) => void;
  onFilesSelect?: (files: FileList) => void;
  accept?: string;
  multiple?: boolean;
  maxSize?: number;
  disabled?: boolean;
  title?: string;
  description?: string;
  icon?: ReactNode;
}

// History Item Props
export interface HistoryItemProps extends BaseComponentProps {
  id: string;
  title: string;
  content?: string;
  timestamp: Date;
  duration?: number;
  fileSize?: number;
  format?: string;
  isSelected?: boolean;
  onSelect?: (id: string) => void;
  onPlay?: (id: string) => void;
  onDelete?: (id: string) => void;
  onExport?: (id: string) => void;
}

// Search Box Props
export interface SearchBoxProps extends Omit<InputProps, 'leftIcon' | 'rightIcon'> {
  onSearch?: (query: string) => void;
  onClear?: () => void;
  showClearButton?: boolean;
  debounceMs?: number;
}

// Animation Props
export interface AnimationProps {
  initial?: any;
  animate?: any;
  exit?: any;
  transition?: any;
  whileHover?: any;
  whileTap?: any;
  whileFocus?: any;
}

// Common event handlers
export type ClickHandler = () => void;
export type ValueChangeHandler<T> = (value: T) => void;
export type FileHandler = (files: FileList) => void;