import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
    // Inline small assets to reduce requests
    assetsInlineLimit: 4096,
  },
  // Dev server proxies API calls to the Rust backend
  server: {
    proxy: {
      '/api': 'http://localhost:8003',
      '/avatars': 'http://localhost:8003',
    },
  },
})
