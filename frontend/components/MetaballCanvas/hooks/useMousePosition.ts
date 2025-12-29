import { useState, useCallback } from 'react';
import { GestureResponderEvent, Dimensions } from 'react-native';

export interface MousePosition {
  x: number;
  y: number;
  normalized: {
    x: number;
    y: number;
  };
}

export function useMousePosition() {
  const [mousePosition, setMousePosition] = useState<MousePosition>({
    x: 0,
    y: 0,
    normalized: { x: 0, y: 0 },
  });

  const { width, height } = Dimensions.get('window');

  const onTouchMove = useCallback(
    (event: GestureResponderEvent) => {
      const { locationX, locationY } = event.nativeEvent;

      const normalized = {
        x: (locationX / width) * 2 - 1,
        y: (locationY / height) * 2 - 1,
      };

      setMousePosition({
        x: locationX,
        y: locationY,
        normalized,
      });
    },
    [width, height]
  );

  return {
    mousePosition,
    onTouchMove,
  };
}
