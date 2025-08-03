import type { Config } from 'tailwindcss'
import sharedConfig from '@repo/tailwind-config/tailwind.config'

const config: Pick<Config, 'presets' | 'content'> = {
  content: [
    './src/app/**/*.tsx',
    './src/components/**/*.tsx',
    '../../packages/ui/src/**/*.{js,ts,jsx,tsx}',
  ],
  presets: [sharedConfig],
}

export default config