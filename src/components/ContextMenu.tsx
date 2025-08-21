import React, { useState, useEffect, useRef, useCallback } from 'react';
import './ContextMenu.css';

export interface ContextMenuItem {
  id: string;
  label: string;
  icon?: string;
  shortcut?: string;
  disabled?: boolean;
  divider?: boolean;
  onClick?: () => void;
}

interface ContextMenuProps {
  items: ContextMenuItem[];
  position: { x: number; y: number };
  onClose: () => void;
  visible: boolean;
}

export const ContextMenu: React.FC<ContextMenuProps> = ({
  items,
  position,
  onClose,
  visible,
}) => {
  const menuRef = useRef<HTMLDivElement>(null);
  const [focusedIndex, setFocusedIndex] = useState(-1);

  // 计算菜单位置，防止超出屏幕边界
  const calculatePosition = useCallback(() => {
    if (!menuRef.current) return position;

    const menu = menuRef.current;
    const rect = menu.getBoundingClientRect();
    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    let { x, y } = position;

    // 防止右边超出
    if (x + rect.width > viewportWidth) {
      x = viewportWidth - rect.width - 8;
    }

    // 防止底部超出
    if (y + rect.height > viewportHeight) {
      y = viewportHeight - rect.height - 8;
    }

    // 防止左边超出
    if (x < 8) {
      x = 8;
    }

    // 防止顶部超出
    if (y < 8) {
      y = 8;
    }

    return { x, y };
  }, [position]);

  // 键盘导航处理
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (!visible) return;

    const validItems = items.filter(item => !item.divider && !item.disabled);

    switch (e.key) {
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
      case 'ArrowDown':
        e.preventDefault();
        setFocusedIndex(prev => {
          const nextIndex = prev + 1;
          return nextIndex >= validItems.length ? 0 : nextIndex;
        });
        break;
      case 'ArrowUp':
        e.preventDefault();
        setFocusedIndex(prev => {
          const nextIndex = prev - 1;
          return nextIndex < 0 ? validItems.length - 1 : nextIndex;
        });
        break;
      case 'Enter':
      case ' ':
        e.preventDefault();
        if (focusedIndex >= 0 && focusedIndex < validItems.length) {
          const item = validItems[focusedIndex];
          item.onClick?.();
          onClose();
        }
        break;
      case 'Tab':
        e.preventDefault();
        break;
    }
  }, [visible, items, focusedIndex, onClose]);

  // 点击外部关闭
  const handleClickOutside = useCallback((e: MouseEvent) => {
    if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
      onClose();
    }
  }, [onClose]);

  useEffect(() => {
    if (visible) {
      document.addEventListener('keydown', handleKeyDown);
      document.addEventListener('mousedown', handleClickOutside);
      setFocusedIndex(-1); // 重置焦点
      
      return () => {
        document.removeEventListener('keydown', handleKeyDown);
        document.removeEventListener('mousedown', handleClickOutside);
      };
    }
  }, [visible, handleKeyDown, handleClickOutside]);

  if (!visible) return null;

  const adjustedPosition = calculatePosition();

  return (
    <div 
      ref={menuRef}
      className="context-menu"
      style={{
        left: adjustedPosition.x,
        top: adjustedPosition.y,
      }}
      role="menu"
      tabIndex={-1}
    >
      {items.map((item, index) => {
        if (item.divider) {
          return <div key={`divider-${index}`} className="context-menu-divider" role="separator" />;
        }

        const validItems = items.filter(item => !item.divider && !item.disabled);
        const validIndex = validItems.indexOf(item);
        const isFocused = focusedIndex === validIndex;

        return (
          <div
            key={item.id}
            className={`context-menu-item ${item.disabled ? 'disabled' : ''} ${isFocused ? 'focused' : ''}`}
            onClick={() => {
              if (!item.disabled) {
                item.onClick?.();
                onClose();
              }
            }}
            onMouseEnter={() => setFocusedIndex(validIndex)}
            role="menuitem"
            tabIndex={-1}
            aria-disabled={item.disabled}
          >
            {item.icon && <span className="menu-item-icon">{item.icon}</span>}
            <span className="menu-item-label">{item.label}</span>
            {item.shortcut && <span className="menu-item-shortcut">{item.shortcut}</span>}
          </div>
        );
      })}
    </div>
  );
};

// Hook for managing context menu state
export const useContextMenu = () => {
  const [contextMenu, setContextMenu] = useState<{
    visible: boolean;
    position: { x: number; y: number };
    items: ContextMenuItem[];
  }>({
    visible: false,
    position: { x: 0, y: 0 },
    items: [],
  });

  const showContextMenu = useCallback((
    event: React.MouseEvent,
    items: ContextMenuItem[]
  ) => {
    event.preventDefault();
    setContextMenu({
      visible: true,
      position: { x: event.clientX, y: event.clientY },
      items,
    });
  }, []);

  const hideContextMenu = useCallback(() => {
    setContextMenu(prev => ({ ...prev, visible: false }));
  }, []);

  return {
    contextMenu,
    showContextMenu,
    hideContextMenu,
  };
};