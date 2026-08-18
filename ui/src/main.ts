import './app.css';
import { mount } from 'svelte';
import App from './App.svelte';

const target = document.getElementById('app') || document.body;

const app = mount(App, {
  target,
});

export default app;
