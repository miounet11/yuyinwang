import React from 'react';
import { createRoot } from 'react-dom/client';
import MacOSVoiceInput from './components/MacOSVoiceInput';

function FloatingInputApp() {
  return <MacOSVoiceInput />;
}

// Render the app
const container = document.getElementById('root');
if (container) {
  const root = createRoot(container);
  root.render(<FloatingInputApp />);
}

export default FloatingInputApp;