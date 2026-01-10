import { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { getSafeId, type Canvas } from './services/backend';
import { CanvasBoard } from './components/CanvasBoard';
import { SettingsWindow } from './components/SettingsWindow';
import { ReactFlowProvider } from '@xyflow/react';



function App() {
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  const [canvasTitle, setCanvasTitle] = useState<string>("Subject");
  const [canvasId, setCanvasId] = useState<string>("");
  const [chatModelId, setChatModelId] = useState<string>("");

  const handleCanvasLoad = (canvas: Canvas) => {
    setCanvasTitle(canvas.title);
    if (canvas.id) {
      setCanvasId(getSafeId(canvas.id));
    }
    if (canvas.chat_model_id) {
      setChatModelId(canvas.chat_model_id);
    }
  };

  return (
    <div className="flex h-screen w-full overflow-hidden bg-background-light dark:bg-background-dark text-slate-900 dark:text-slate-100 transition-colors duration-200">
      <Sidebar
        onOpenSettings={() => setIsSettingsOpen(true)}
        title={canvasTitle}
        canvasId={canvasId}
        initialModelId={chatModelId}
      />
      {isSettingsOpen && <SettingsWindow onClose={() => setIsSettingsOpen(false)} />}
      <ReactFlowProvider>
        <CanvasBoard onCanvasLoad={handleCanvasLoad} />
      </ReactFlowProvider>
    </div>
  );
}

export default App;
