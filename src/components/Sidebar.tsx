import React, { useState, useRef, useEffect } from 'react';
import { ModelSelector } from './ModelSelector';

import { GraphAPI } from '../services/backend';

interface SidebarProps {
    onOpenSettings?: () => void;
    title?: string;
    canvasId?: string;
    initialModelId?: string;
}

export const Sidebar: React.FC<SidebarProps> = ({ onOpenSettings, title = "Subject", canvasId, initialModelId }) => {
    const [isTruncated, setIsTruncated] = useState(false);
    const [currentModel, setCurrentModel] = useState(initialModelId || "");
    const [query, setQuery] = useState("");
    const [isShaking, setIsShaking] = useState(false);
    const titleRef = useRef<HTMLHeadingElement>(null);

    useEffect(() => {
        if (initialModelId) {
            setCurrentModel(initialModelId);
        }
    }, [initialModelId]);

    useEffect(() => {
        const checkTruncation = () => {
            if (titleRef.current) {
                setIsTruncated(titleRef.current.scrollWidth > titleRef.current.clientWidth);
            }
        };

        checkTruncation();
        window.addEventListener('resize', checkTruncation);
        return () => window.removeEventListener('resize', checkTruncation);
    }, [title]);

    const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
            e.preventDefault();
            if (!currentModel) {
                setIsShaking(true);
                setTimeout(() => setIsShaking(false), 500);
                return;
            }
            if (query.trim()) {
                console.log("Unified Query Submitted:", query);
                // TODO: Implement actual submission logic here
                setQuery("");
            }
        }
    };

    return (
        <aside className="w-80 flex flex-col border-r border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark shadow-sm z-20 flex-shrink-0">
            <div className="h-12 px-4 flex items-center justify-between border-b border-border-light dark:border-border-dark">
                <button className="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-500 dark:text-slate-400 transition-colors">
                    <span className="material-icons-round">menu</span>
                </button>
                <div className="flex-1 px-4 min-w-0 flex justify-start">
                    <h1
                        ref={titleRef}
                        className="text-base font-semibold text-slate-800 dark:text-white truncate cursor-default"
                        title={isTruncated ? title : undefined}
                    >
                        {title}
                    </h1>
                </div>
                <div className="flex items-center gap-2">
                    <button onClick={onOpenSettings} className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-400 dark:text-slate-500 transition-colors">
                        <span className="material-icons-round text-xl">settings</span>
                    </button>
                    <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center text-primary font-bold overflow-hidden cursor-pointer hover:ring-2 hover:ring-primary hover:ring-offset-1 dark:ring-offset-slate-900 transition-all">
                        <span className="material-icons-round text-sm">person</span>
                    </div>
                </div>
            </div>

            <div className="flex-1 overflow-y-auto custom-scrollbar p-4 flex flex-col items-center justify-center text-center">
                <div className="max-w-[200px] space-y-4 opacity-50">
                    <span className="material-icons-round text-4xl text-primary/40">smart_toy</span>
                    <p className="text-lg font-medium text-slate-400 dark:text-slate-500">How can I help?</p>
                    <p className="text-sm text-slate-400 dark:text-slate-500">Ask questions across all your interconnected nodes.</p>
                </div>
            </div>

            <div className="p-4 border-t border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark">
                <div className="relative group">
                    <div className="absolute -top-10 left-0 right-0 flex justify-center pb-2 opacity-0 group-hover:opacity-100 transition-opacity">
                        <span className="text-xs bg-slate-200 dark:bg-slate-700 text-slate-600 dark:text-slate-300 px-2 py-1 rounded-full">Unified Query</span>
                    </div>
                    <div className="bg-slate-100 dark:bg-slate-900 rounded-lg p-3 border border-transparent focus-within:border-primary focus-within:ring-1 focus-within:ring-primary transition-all shadow-sm">
                        <textarea
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            onKeyDown={handleKeyDown}
                            className="w-full bg-transparent border-none p-0 text-sm focus:ring-0 resize-none h-12 placeholder-slate-400 dark:placeholder-slate-500 text-slate-800 dark:text-slate-200"
                            placeholder="Ask anything..."
                        ></textarea>
                        <div className="flex items-center justify-between mt-2">
                            <button className="p-1.5 rounded-full hover:bg-slate-200 dark:hover:bg-slate-800 text-slate-500 dark:text-slate-400 transition-colors">
                                <span className="material-icons-round text-lg">add</span>
                            </button>
                            <div className="flex items-center gap-2">
                                <button className="p-1.5 rounded-full hover:bg-slate-200 dark:hover:bg-slate-800 text-slate-500 dark:text-slate-400 transition-colors">
                                    <span className="material-icons-round text-lg">tune</span>
                                </button>
                                <button className="p-1.5 rounded-full bg-primary text-white shadow-md hover:bg-indigo-700 transition-colors">
                                    <span className="material-icons-round text-lg">add</span>
                                </button>
                            </div>
                        </div>
                    </div>
                    <div className="mt-2 text-slate-400 dark:text-slate-500 px-1">
                        <ModelSelector
                            currentModel={currentModel}
                            onModelSelect={async (model) => {
                                setCurrentModel(model);
                                if (canvasId) {
                                    try {
                                        await GraphAPI.updateCanvasChatModel(canvasId, model);
                                    } catch (e) {
                                        console.error("Failed to update canvas chat model", e);
                                    }
                                }
                            }}
                            className={isShaking ? 'animate-shake' : ''}
                        />
                    </div>
                </div>
            </div>
        </aside>
    );
};
