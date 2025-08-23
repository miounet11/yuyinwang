/**
 * Spokenly Design System - Component Library
 * Central export for all UI components with proper TypeScript support
 */

// Core Layout Components
export { SpokenlyLayout } from './SpokenlyLayout';
export { SpokemlySidebar } from './SpokemlySidebar';
export { SpokenlyContent } from './SpokenlyContent';

// Navigation Components
export { SpokenlyNavItem } from './SpokenlyNavItem';
export { SpokenlyNavSection } from './SpokenlyNavSection';

// Form Components
export { SpokenlyButton } from './SpokenlyButton';
export { SpokenlyInput } from './SpokenlyInput';
export { SpokenlySwitch } from './SpokenlySwitch';
export { SpokenlySelect } from './SpokenlySelect';

// Card Components
export { SpokenlyCard } from './SpokenlyCard';
export { SpokenlyCardHeader, SpokenlyCardBody, SpokenlyCardFooter } from './SpokenlyCard';

// Content Components
export { SpokenlyTag } from './SpokenlyTag';
export { SpokenlyModelCard } from './SpokenlyModelCard';
export { SpokenlyUploadArea } from './SpokenlyUploadArea';
export { SpokenlyHistoryItem } from './SpokenlyHistoryItem';
export { SpokenlySearchBox } from './SpokenlySearchBox';

// Type exports
export type * from './types';

// Re-export specific types that are commonly used
export type { 
  ButtonProps,
  CardProps,
  LayoutProps,
  SidebarProps,
  ContentProps,
  NavItemProps,
  InputProps,
  SwitchProps,
  SelectProps,
  TagProps,
  ButtonVariant,
  ButtonSize,
  Size 
} from './types';