import { useState, useEffect, useRef, useMemo } from 'react';

interface UseVirtualScrollOptions {
  itemCount: number;
  itemHeight: number;
  containerHeight: number;
  overscan?: number;
}

interface VirtualScrollResult {
  virtualItems: Array<{
    index: number;
    start: number;
  }>;
  totalHeight: number;
  scrollToIndex: (index: number) => void;
}

/**
 * Custom hook for virtual scrolling optimization
 * Only renders visible items + overscan buffer
 */
export function useVirtualScroll({
  itemCount,
  itemHeight,
  containerHeight,
  overscan = 3,
}: UseVirtualScrollOptions): VirtualScrollResult {
  const [scrollTop, setScrollTop] = useState(0);
  const scrollRef = useRef<HTMLDivElement>(null);

  const { startIndex, endIndex, virtualItems } = useMemo(() => {
    const start = Math.floor(scrollTop / itemHeight);
    const visibleCount = Math.ceil(containerHeight / itemHeight);

    const startIdx = Math.max(0, start - overscan);
    const endIdx = Math.min(itemCount - 1, start + visibleCount + overscan);

    const items = [];
    for (let i = startIdx; i <= endIdx; i++) {
      items.push({
        index: i,
        start: i * itemHeight,
      });
    }

    return {
      startIndex: startIdx,
      endIndex: endIdx,
      virtualItems: items,
    };
  }, [scrollTop, itemHeight, containerHeight, itemCount, overscan]);

  const totalHeight = itemCount * itemHeight;

  const scrollToIndex = (index: number) => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = index * itemHeight;
    }
  };

  useEffect(() => {
    const element = scrollRef.current;
    if (!element) return;

    const handleScroll = () => {
      setScrollTop(element.scrollTop);
    };

    element.addEventListener('scroll', handleScroll, { passive: true });
    return () => element.removeEventListener('scroll', handleScroll);
  }, []);

  return {
    virtualItems,
    totalHeight,
    scrollToIndex,
  };
}
