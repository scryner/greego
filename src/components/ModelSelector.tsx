import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ModelSelectorProps {
    currentModel: string;
    onModelSelect?: (model: string) => void;
}

export const ModelSelector = ({ currentModel, onModelSelect }: ModelSelectorProps) => {
    const [isOpen, setIsOpen] = useState(false);
    const [availableModels, setAvailableModels] = useState<string[]>([]);
    const containerRef = useRef<HTMLDivElement>(null);

    const handleToggle = async () => {
        if (!isOpen) {
            try {
                const models = await invoke<string[]>('get_llm_available_models');
                setAvailableModels(models);
            } catch (error) {
                console.error("Failed to fetch models:", error);
            }
        }
        setIsOpen(!isOpen);
    };

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
                setIsOpen(false);
            }
        };

        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, []);

    return (
        <div className="relative inline-block nodrag nopan nowheel" ref={containerRef}>
            <button
                onClick={handleToggle}
                className={`
                    px-2 py-1 -ml-2 rounded-lg text-xs font-medium transition-colors duration-200 flex items-center gap-1
                    ${isOpen
                        ? 'bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-200'
                        : 'text-slate-400 dark:text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 hover:text-slate-600 dark:hover:text-slate-300'
                    }
                `}
            >
                {currentModel || "Select model"}
                <span className="material-icons-round text-[10px] opacity-70">expand_more</span>
            </button>

            {isOpen && (
                <div
                    className="absolute left-0 bottom-full mb-2 w-48 bg-white dark:bg-slate-800 rounded-lg shadow-xl border border-slate-100 dark:border-slate-700 z-[60] animate-in fade-in zoom-in-95 duration-100 origin-bottom-left cursor-default nowheel"
                    onWheel={(e) => e.stopPropagation()}
                >
                    <div className="relative bg-white dark:bg-slate-800 rounded-lg overflow-hidden flex flex-col max-h-48 nowheel">
                        <div className="overflow-y-auto custom-scrollbar p-1 nodrag nopan nowheel">
                            {availableModels.length > 0 ? (
                                availableModels.map((model) => (
                                    <button
                                        key={model}
                                        onClick={() => {
                                            if (onModelSelect) onModelSelect(model);
                                            setIsOpen(false);
                                        }}
                                        className={`
                                            w-full text-left px-3 py-2 text-xs rounded-md transition-colors truncate
                                            ${currentModel === model
                                                ? 'bg-primary/10 text-primary font-semibold'
                                                : 'text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700'
                                            }
                                        `}
                                    >
                                        {model}
                                    </button>
                                ))
                            ) : (
                                <div className="px-3 py-2 text-xs text-slate-400 italic">No models found</div>
                            )}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
