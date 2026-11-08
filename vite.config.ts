import { configDefaults, defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  test: {
    exclude: [...configDefaults.exclude, 'target/**', 'src-tauri/target/**', '.qualification/**'],
    environment: 'jsdom', setupFiles: ['./src/test-setup.ts'],
    coverage: { provider: 'v8', reporter: ['text', 'lcov'], include: ['src/**/*.{ts,tsx}'], exclude: ['src/main.tsx', 'src/**/*.test.*', 'src/test-setup.ts'] },
  },
});
