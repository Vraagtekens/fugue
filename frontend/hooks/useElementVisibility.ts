import { useEffect, useState, RefObject } from 'react';

/**
 * Custom hook to detect when an element is visible in the viewport
 * @param elementRef Reference to the HTML element to track
 * @param rootMargin Optional margin around the viewport (default: '0px')
 * @returns Boolean indicating if the element is visible
 */
export function useElementVisibility<T extends HTMLElement>(
  elementRef: RefObject<T>,
  rootMargin: string = '0px'
): boolean {
  const [isVisible, setIsVisible] = useState(true); // Default to true for SSR and initial render

  useEffect(() => {
    const currentElement = elementRef.current;
    if (!currentElement || typeof IntersectionObserver === 'undefined') return;

    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsVisible(entry.isIntersecting);
      },
      { rootMargin }
    );

    observer.observe(currentElement);

    return () => {
      observer.disconnect();
    };
  }, [elementRef, rootMargin]);

  return isVisible;
}
