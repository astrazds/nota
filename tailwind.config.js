/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: [
    "./index.html",
    "./src/**/*.rs",
  ],
  theme: {
    extend: {
      fontFamily: {
        sans: ['"Source Sans 3 Variable"', 'ui-sans-serif', 'system-ui', 'sans-serif'],
        mono: ['"Source Code Pro Variable"', 'ui-monospace', 'SFMono-Regular', 'Menlo', 'monospace'],
      },
      colors: {
        apple: {
          yellow: '#FFB340',
          dark: {
            bg: '#1C1C1E',
            sidebar: '#2C2C2E',
            border: '#3A3A3C',
            text: '#FFFFFF',
          },
          gray: {
            100: '#F5F5F7',
            200: '#E8E8ED',
            300: '#D2D2D7',
          }
        }
      },
      typography: (theme) => ({
        DEFAULT: {
          css: {
            '--tw-prose-body': theme('colors.gray.700'),
            '--tw-prose-headings': theme('colors.gray.900'),
            '--tw-prose-links': theme('colors.gray.900'),
            '--tw-prose-bold': theme('colors.gray.900'),
            '--tw-prose-counters': theme('colors.gray.500'),
            '--tw-prose-bullets': theme('colors.gray.500'),
            '--tw-prose-hr': theme('colors.gray.200'),
            '--tw-prose-quotes': theme('colors.gray.900'),
            '--tw-prose-quote-borders': theme('colors.yellow.500'),
            '--tw-prose-captions': theme('colors.gray.500'),
            '--tw-prose-code': theme('colors.gray.900'),
            '--tw-prose-pre-code': theme('colors.gray.200'),
            '--tw-prose-pre-bg': theme('colors.gray.800'),
            '--tw-prose-th-borders': theme('colors.gray.300'),
            '--tw-prose-td-borders': theme('colors.gray.200'),
          },
        },
        invert: {
          css: {
            '--tw-prose-body': theme('colors.gray.300'),
            '--tw-prose-headings': theme('colors.white'),
            '--tw-prose-links': theme('colors.white'),
            '--tw-prose-bold': theme('colors.white'),
            '--tw-prose-counters': theme('colors.gray.500'),
            '--tw-prose-bullets': theme('colors.gray.500'),
            '--tw-prose-hr': theme('colors.gray.700'),
            '--tw-prose-quotes': theme('colors.gray.100'),
            '--tw-prose-quote-borders': theme('colors.yellow.500'),
            '--tw-prose-captions': theme('colors.gray.500'),
            '--tw-prose-code': theme('colors.gray.100'),
            '--tw-prose-pre-code': theme('colors.gray.300'),
            '--tw-prose-pre-bg': theme('colors.gray.800'),
            '--tw-prose-th-borders': theme('colors.gray.600'),
            '--tw-prose-td-borders': theme('colors.gray.700'),
          },
        },
      }),
    },
  },
  plugins: [
    require('@tailwindcss/typography'),
  ],
}
