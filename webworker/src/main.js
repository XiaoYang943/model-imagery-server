import { createApp } from 'vue';
import { router } from './router';
import './style.css';
import 'element-plus/theme-chalk/index.css';
import ElementPlus from 'element-plus';
import App from './App.vue';

createApp(App).use(router).use(ElementPlus).mount('#app');
