import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import AutoImport from 'unplugin-auto-import/vite';
import Pages from 'vite-plugin-pages';
import { viteExternalsPlugin } from 'vite-plugin-externals'
export default defineConfig({
  base: "./",
  plugins: [
    vue(),
    Pages({
      dirs: 'src/views',
      extendRoute(route) {
        if (route.path === "/") return { ...route, redirect: "main" }
      }
    }),
    AutoImport({
      imports: ['vue', 'vue-router'], // 自动导入vue和vue-router相关函数
      dts: 'src/auto-import.d.ts', // 生成 `auto-import.d.ts` 全局声明
    }),
    viteExternalsPlugin({
      cesium: 'Cesium',
    }),
  ],
  resolve: {
    alias: {
      '@': '/src',
    },
  },
  build: {
    assetsDir: "modelImageryServer",
    lib: {
      entry: 'src/lib/modelImageryServer.js', // 设置入口文件
      formats: ["umd"],
      name: "modelImageryServer",
      fileName: (format) => `modelImageryServer/modelImageryServer.js` // 打包后的文件名
    },
  },
  worker: {
    // 输出格式，默认值为 'iife'
    format: 'es',
    // 应用于 worker bundle 的 Vite 插件
    plugins: [],
    // Rollup 选项来构建 worker bundle
    rollupOptions: {},
  },
  define: {
    global: {},
    'process.env': process.env
  },
  server: {
    hmr: false,
    port: 5174,
    host: "0.0.0.0"
  },
  test: {
    browser: {
      enabled: false,
      name: 'edge', // browser name is required
    },
  },
});