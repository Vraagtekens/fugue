import { useState, useCallback } from 'react';
import { LayoutChangeEvent } from 'react-native';

interface Size {
  width: number;
  height: number;
  aspect: number;
}

export function useContainerSize() {
  const [size, setSize] = useState<Size>({
    width: 0,
    height: 0,
    aspect: 1,
  });

  const onLayout = useCallback((event: LayoutChangeEvent) => {
    const { width, height } = event.nativeEvent.layout;

    setSize({
      width,
      height,
      aspect: width / height || 1,
    });
  }, []);

  return { onLayout, size };
}
