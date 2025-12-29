const { hairlineWidth } = require('nativewind/theme');

/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: ['./app/**/*.{ts,tsx}', './components/**/*.{ts,tsx}'],
  presets: [require('nativewind/preset')],
  theme: {
    extend: {
      colors: {
        border: 'hsl(var(--border))',
        input: 'hsl(var(--input))',
        ring: 'hsl(var(--ring))',
        background: 'hsl(var(--background))',
        foreground: 'hsl(var(--foreground))',
        primary: {
          DEFAULT: 'hsl(var(--primary))',
          foreground: 'hsl(var(--primary-foreground))',
        },
        secondary: {
          DEFAULT: 'hsl(var(--secondary))',
          foreground: 'hsl(var(--secondary-foreground))',
        },
        destructive: {
          DEFAULT: 'hsl(var(--destructive))',
          foreground: 'hsl(var(--destructive-foreground))',
        },
        muted: {
          DEFAULT: 'hsl(var(--muted))',
          foreground: 'hsl(var(--muted-foreground))',
        },
        accent: {
          DEFAULT: 'hsl(var(--accent))',
          foreground: 'hsl(var(--accent-foreground))',
        },
        popover: {
          DEFAULT: 'hsl(var(--popover))',
          foreground: 'hsl(var(--popover-foreground))',
        },
        card: {
          DEFAULT: 'hsl(var(--card))',
          foreground: 'hsl(var(--card-foreground))',
        },
      },
      borderRadius: {
        lg: 'var(--radius)',
        md: 'calc(var(--radius) - 2px)',
        sm: 'calc(var(--radius) - 4px)',
      },
      borderWidth: {
        hairline: hairlineWidth(),
      },
      keyframes: {
        'accordion-down': {
          from: { height: '0' },
          to: { height: 'var(--radix-accordion-content-height)' },
        },
        'accordion-up': {
          from: { height: 'var(--radix-accordion-content-height)' },
          to: { height: '0' },
        },
      },
      animation: {
        'accordion-down': 'accordion-down 0.2s ease-out',
        'accordion-up': 'accordion-up 0.2s ease-out',
      },

      backgroundImage: {
        // Gradient presets
        'gradient-green-blue':
          'linear-gradient(to left, var(--color-green-bright), var(--color-green-soft), var(--color-blue-soft))',
        'gradient-blues':
          'linear-gradient(to left, var(--color-blue-bright), var(--color-blue-medium), var(--color-blue-dark))',
        'gradient-purple':
          'linear-gradient(to right, var(--color-purple), var(--color-purple-blue), var(--color-blue-sky))',

        // Diagonal variations
        'gradient-green-blue-diagonal':
          'linear-gradient(135deg, var(--color-teal), var(--color-green-soft), var(--color-blue-soft))',
        'gradient-blues-diagonal':
          'linear-gradient(135deg, var(--color-blue-bright), var(--color-blue-medium), var(--color-blue-dark))',
        'gradient-purple-diagonal':
          'linear-gradient(135deg, var(--color-purple), var(--color-purple-blue), var(--color-blue-sky))',

        'gradient-orange':
          'linear-gradient(to right, var(--color-orange), var(--color-white), var(--color-white))',
      },
    },
  },
  future: {
    hoverOnlyWhenSupported: true,
  },
  plugins: [require('tailwindcss-animate')],
};
