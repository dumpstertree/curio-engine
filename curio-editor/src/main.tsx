import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './App.css';

// src/main.tsx
window.addEventListener('error', (e) => {
    document.body.innerHTML = `
    <div style="color:red;padding:20px;font-family:monospace;white-space:pre-wrap">
      ${e.message}\n\n${e.error?.stack ?? ''}
    </div>
  `;
});

window.addEventListener('unhandledrejection', (e) => {
    document.body.innerHTML = `
    <div style="color:red;padding:20px;font-family:monospace;white-space:pre-wrap">
      Unhandled promise rejection:\n${e.reason}
    </div>
  `;
});
ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    <React.StrictMode>
        <App />
    </React.StrictMode>
);