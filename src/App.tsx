import { Sidebar } from './components/Sidebar';
import { CanvasBoard } from './components/CanvasBoard';
import { ReactFlowProvider } from '@xyflow/react';

import { useEffect } from 'react';
import { GraphAPI } from './services/backend';

function App() {
  useEffect(() => {
    GraphAPI.loadGraph()
      .then(([nodes, edges]) => {
        console.log('Graph loaded:', { nodes, edges });
      })
      .catch((err) => {
        console.error('Failed to load graph:', err);
      });
  }, []);

  return (
    <div className="flex h-screen w-full overflow-hidden bg-background-light dark:bg-background-dark text-slate-900 dark:text-slate-100 transition-colors duration-200">
      <Sidebar />
      <ReactFlowProvider>
        <CanvasBoard />
      </ReactFlowProvider>
    </div>
  );
}

export default App;
