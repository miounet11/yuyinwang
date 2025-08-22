/**
 * Spokenly Design System - Component Library
 * Centralized exports for all UI components
 * 
 * @version 1.0.0
 * @author Recording King Team
 */

// Type definitions
export * from './types';

// Layout Components
export { SpokenlyLayout } from './SpokenlyLayout';
export { SpokemlySidebar } from './SpokemlySidebar';
export { SpokenlyContent } from './SpokenlyContent';

// Navigation Components
export { SpokenlyNavItem } from './SpokenlyNavItem';
export { SpokenlyNavSection } from './SpokenlyNavSection';

// Basic Components
export { SpokenlyButton } from './SpokenlyButton';
export { 
  SpokenlyCard, 
  SpokenlyCardHeader, 
  SpokenlyCardBody, 
  SpokenlyCardFooter 
} from './SpokenlyCard';
export { SpokenlyInput } from './SpokenlyInput';

// Form Components
export { SpokenlySwitch } from './SpokenlySwitch';
export { SpokenlySelect } from './SpokenlySelect';
export { SpokenlyTag } from './SpokenlyTag';

// Specialized Components
export { SpokenlyModelCard } from './SpokenlyModelCard';
export { SpokenlyUploadArea } from './SpokenlyUploadArea';

// Interaction Components
export { SpokenlyHistoryItem } from './SpokenlyHistoryItem';
export { SpokenlySearchBox } from './SpokenlySearchBox';

// Component Groups for easier imports
export const LayoutComponents = {
  Layout: SpokenlyLayout,
  Sidebar: SpokemlySidebar,
  Content: SpokenlyContent
};

export const NavigationComponents = {
  NavItem: SpokenlyNavItem,
  NavSection: SpokenlyNavSection
};

export const FormComponents = {
  Button: SpokenlyButton,
  Input: SpokenlyInput,
  Switch: SpokenlySwitch,
  Select: SpokenlySelect,
  Tag: SpokenlyTag
};

export const CardComponents = {
  Card: SpokenlyCard,
  CardHeader: SpokenlyCardHeader,
  CardBody: SpokenlyCardBody,
  CardFooter: SpokenlyCardFooter
};

export const SpecializedComponents = {
  ModelCard: SpokenlyModelCard,
  UploadArea: SpokenlyUploadArea,
  HistoryItem: SpokenlyHistoryItem,
  SearchBox: SpokenlySearchBox
};

// Version information
export const SPOKENLY_VERSION = '1.0.0';
export const DESIGN_SYSTEM_VERSION = '1.0.0';