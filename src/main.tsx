import React from 'react';
import ReactDOM from 'react-dom/client';
import SpokenlyApp from './components/SpokenlyApp';
import './styles/spokenly-design-system.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <SpokenlyApp />
  </React.StrictMode>
);