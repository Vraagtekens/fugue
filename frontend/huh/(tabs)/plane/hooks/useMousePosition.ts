import { isMobile } from './deviceDetection';
import { useState, useEffect } from 'react';

interface MousePosition {
  x: number;
  y: number;
  normalized: {
    x: number;
    y: number;
  };
}

export function useMousePosition(): MousePosition {
  const [mousePosition, setMousePosition] = useState<MousePosition>({
    x: 0,
    y: 0,
    normalized: { x: 0, y: 0 },
  });

  useEffect(() => {
    // Skip for mobile devices
    if (isMobile()) return;

    const updateMousePosition = (e: MouseEvent): void => {
      const x = e.clientX;
      const y = e.clientY;

      // Normalize coordinates to -1 to 1 range
      const normalized = {
        x: (x / window.innerWidth) * 2 - 1,
        // y: -(y / window.innerHeight) * 2 + 1,
        y: (-(y / window.innerHeight) * 2 + 1) * -1,
      };

      setMousePosition({ x, y, normalized });
    };

    window.addEventListener('mousemove', updateMousePosition);

    return () => {
      window.removeEventListener('mousemove', updateMousePosition);
    };
  }, []);

  return mousePosition;
}
