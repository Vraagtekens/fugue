export const isMobile = (): boolean => {
  if (typeof window === 'undefined') return false // SSR check
  return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
}

export const getDevicePixelRatio = (): number => {
  if (typeof window === 'undefined') return 1 // SSR check
  return Math.min(window.devicePixelRatio, 2) // Cap at 2 for performance
}

// Get performance tier for shader complexity adjustments
export const getPerformanceTier = (): 'low' | 'medium' | 'high' => {
  // === SSR SAFEGUARD ===
  if (typeof window === 'undefined' || typeof navigator === 'undefined') {
    // on the server, just pick a safe default
    return 'high'
  }

  const mobile = isMobile()
  const dpr = getDevicePixelRatio()
  const cores = navigator.hardwareConcurrency ?? 4 // fallback if undefined

  if (mobile && dpr < 2) return 'low'
  if (mobile || (dpr < 2 && cores <= 4)) return 'medium'
  return 'high'
}
