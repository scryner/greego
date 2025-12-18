import { useState, useRef, useEffect } from 'react';

interface ModelSelectorProps {
    currentModel: string;
    onModelSelect?: (model: string) => void;
}

const AVAILABLE_MODELS = [
    'lms/gpt-oss-120b',
    'lms/claude-3-opus',
    'lms/gpt-4-turbo',
    'lms/mistral-large',
    'lms/gemini-1.5-pro',
    'lms/llama-3-70b',
    'lms/stable-code-3b'
];

export const ModelSelector = ({ currentModel: initialModel, onModelSelect }: ModelSelectorProps) => {
    const [isOpen, setIsOpen] = useState(false);
    const [selectedModel, setSelectedModel] = useState(initialModel);
    const containerRef = useRef<HTMLDivElement>(null);

    // Sync state if prop changes
    useEffect(() => {
        setSelectedModel(initialModel);
    }, [initialModel]);

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
                onClick={() => setIsOpen(!isOpen)}
                className={`
                    px-2 py-1 -ml-2 rounded-lg text-xs font-medium transition-colors duration-200 flex items-center gap-1
                    ${isOpen
                        ? 'bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-200'
                        : 'text-slate-400 dark:text-slate-500 hover:bg-slate-100 dark:hover:bg-slate-800 hover:text-slate-600 dark:hover:text-slate-300'
                    }
                `}
            >
                {selectedModel}
                <span className="material-icons-round text-[10px] opacity-70">expand_more</span>
            </button>

            {isOpen && (
                <div
                    className="absolute left-0 bottom-full mb-2 w-48 bg-white dark:bg-slate-800 rounded-lg shadow-xl border border-slate-100 dark:border-slate-700 z-[60] animate-in fade-in zoom-in-95 duration-100 origin-bottom-left cursor-default nowheel"
                    onWheel={(e) => e.stopPropagation()} // Stop propagation to prevent canvas zoom
                >
                    {/* Arrow pointing down (since it pops up) */}
                    {/* Note: The requirement said "similar to kebab menu". Kebab menu usually pops down.
                         But footer is at the bottom, so popping UP might be safer to avoid clipping?
                         Let's try popping DOWN first as kebab usually does, but check if it clips.
                         Since it's a "New chat" node, it's 100,100. There is space below.
                         Let's stick to POPUP behavior (appearing on top or bottom).
                         If the previous menu popped down, let's try popping DOWN for consistency unless space is issues.
                         However, "Pop-up menu" implies popping... up? No, standard dropdown.
                         Let's use bottom-full (pop UP) because it's in the footer?
                         Actually, standard dropdowns go down.
                         Let's change to top-full (ModelSelector is in footer, so top-full goes DOWN outside the node).
                         This prevents covering the content.
                     */}
                    <div className="mb-2 absolute -top-1.5 left-4 w-3 h-3 bg-white dark:bg-slate-800 transform rotate-45 border-t border-l border-slate-100 dark:border-slate-700 z-10"></div>

                    {/* Actually, if we use top-full, the arrow should be at the top.
                        Wait, if it's top-full, the menu is BELOW the button.
                        So arrow is at TOP of menu.
                    */}

                    <div className="relative bg-white dark:bg-slate-800 rounded-lg overflow-hidden flex flex-col max-h-48 nowheel">
                        <div className="overflow-y-auto custom-scrollbar p-1 nodrag nopan nowheel">
                            {AVAILABLE_MODELS.map((model) => (
                                <button
                                    key={model}
                                    onClick={() => {
                                        setSelectedModel(model);
                                        if (onModelSelect) onModelSelect(model);
                                        setIsOpen(false);
                                    }}
                                    className={`
                                        w-full text-left px-3 py-2 text-xs rounded-md transition-colors
                                        ${selectedModel === model
                                            ? 'bg-primary/10 text-primary font-semibold'
                                            : 'text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700'
                                        }
                                    `}
                                >
                                    {model}
                                </button>
                            ))}
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
