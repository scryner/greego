import { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { CanvasBoard } from './components/CanvasBoard';
import { SettingsWindow } from './components/SettingsWindow';
import { ReactFlowProvider } from '@xyflow/react';



function App() {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);

  return (
    <div className="flex h-screen w-full overflow-hidden bg-background-light dark:bg-background-dark text-slate-900 dark:text-slate-100 transition-colors duration-200">
      <Sidebar onOpenSettings={() => setIsSettingsOpen(true)} />
      {isSettingsOpen && <SettingsWindow onClose={() => setIsSettingsOpen(false)} />}
      <ReactFlowProvider>
        <CanvasBoard />
      </ReactFlowProvider>
    </div>
  );
}

export default App;
