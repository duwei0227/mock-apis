import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  base: '/mock/',
  plugins: [
    vue(),
    tailwindcss(),
  ],
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
  server: {
    proxy: {
      '/mock/api': 'http://localhost:9999',
      '/mock/ws': { target: 'ws://localhost:9999', ws: true },
    },
  },
})
