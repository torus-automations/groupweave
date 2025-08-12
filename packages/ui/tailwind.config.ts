import type { Config } from 'tailwindcss'
import sharedConfig from '@repo/tailwind-config/tailwind.config'

const config: Config = {
  content: ['./src/**/*.tsx'],
  presets: [sharedConfig as Config],
}

export default config