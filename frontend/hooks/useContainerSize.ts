import { useState, useEffect, useRef, RefObject } from 'react';

interface Size {
  width: number;
  height: number;
  aspect: number;
}

export function useContainerSize(): [RefObject<HTMLDivElement>, Size] {
  const containerRef = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState<Size>({ width: 0, height: 0, aspect: 1 });

  useEffect(() => {
    if (!containerRef.current) return;

    const updateSize = (entries: ResizeObserverEntry[]): void => {
      const entry = entries[0];
      if (!entry) return;
      const { width, height } = entry.contentRect;
      setSize({
        width,
        height,
        aspect: width / height || 1, // Fallback to 1 if division results in NaN
      });
    };

    // Use ResizeObserver for more efficient size tracking than window resize
    const resizeObserver = new ResizeObserver(updateSize);
    resizeObserver.observe(containerRef.current);

    // Initial size calculation
    setSize({
      width: containerRef.current.clientWidth,
      height: containerRef.current.clientHeight,
      aspect: containerRef.current.clientWidth / containerRef.current.clientHeight || 1,
    });

    return () => resizeObserver.disconnect();
  }, []);

  return [containerRef, size] as [RefObject<HTMLDivElement>, Size];
}
