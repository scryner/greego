import { Sidebar } from './components/Sidebar';
import { CanvasBoard } from './components/CanvasBoard';

function App() {
  return (
    <div className="flex h-screen w-full overflow-hidden bg-background-light dark:bg-background-dark text-slate-900 dark:text-slate-100 transition-colors duration-200">
      <Sidebar />
      <CanvasBoard />
    </div>
  );
}

export default App;
