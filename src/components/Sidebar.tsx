import React, { useState, useRef, useEffect } from 'react';
import { ModelSelector } from './ModelSelector';

import { GraphAPI } from '../services/backend';

interface SidebarProps {
    onOpenSettings?: () => void;
    title?: string;
    canvasId?: string;
    initialModelId?: string;
}

interface Message {
    role: 'user' | 'assistant';
    content: string;
}

export const Sidebar: React.FC<SidebarProps> = ({ onOpenSettings, title = "Subject", canvasId, initialModelId }) => {
    const [isTruncated, setIsTruncated] = useState(false);
    const [currentModel, setCurrentModel] = useState(initialModelId || "");
    const [query, setQuery] = useState("");
    const [isShaking, setIsShaking] = useState(false);
    const [messages, setMessages] = useState<Message[]>([]);
    const titleRef = useRef<HTMLHeadingElement>(null);
    const messagesEndRef = useRef<HTMLDivElement>(null);

    const scrollToBottom = () => {
        messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    };

    useEffect(() => {
        scrollToBottom();
    }, [messages]);

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

    const unlistenersRef = useRef<(() => void)[]>([]);

    useEffect(() => {
        let isEffectActive = true;

        const setupListeners = async () => {
            const { listen } = await import('@tauri-apps/api/event');

            const deltaUnlisten = await listen('unified-query-delta', (event: any) => {
                setMessages(prev => {
                    const newMessages = [...prev];
                    if (newMessages.length === 0) return prev;

                    const lastMsgIndex = newMessages.length - 1;
                    const lastMessage = { ...newMessages[lastMsgIndex] };

                    if (lastMessage.role === 'assistant') {
                        lastMessage.content += event.payload.content;
                        newMessages[lastMsgIndex] = lastMessage;
                        return newMessages;
                    }
                    return prev;
                });
            });

            const doneUnlisten = await listen('unified-query-done', () => {
                console.log('Unified Query Done');
            });

            const errorUnlisten = await listen('unified-query-error', (event: any) => {
                console.error('Unified Query Error:', event.payload.error);
                setMessages(prev => [...prev, { role: 'assistant', content: `Error: ${event.payload.error}` }]);
            });

            if (isEffectActive) {
                unlistenersRef.current.push(deltaUnlisten, doneUnlisten, errorUnlisten);
            } else {
                deltaUnlisten();
                doneUnlisten();
                errorUnlisten();
            }
        };

        setupListeners();

        return () => {
            isEffectActive = false;
            unlistenersRef.current.forEach(u => u());
            unlistenersRef.current = [];
        };
    }, []);

    const handleKeyDown = async (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
        if (e.key === 'Enter' && !e.shiftKey && !e.nativeEvent.isComposing) {
            e.preventDefault();
            console.log("Unified Query KeyDown: Enter pressed. CurrentModel:", currentModel);

            if (!currentModel) {
                console.warn("Unified Query: No model selected. Shaking.");
                setIsShaking(true);
                setTimeout(() => setIsShaking(false), 500);
                return;
            }
            if (query.trim()) {
                const userQuery = query;
                setQuery(""); // Clear immediately
                setMessages(prev => [
                    ...prev,
                    { role: 'user', content: userQuery },
                    { role: 'assistant', content: '' } // Placeholder for streaming response
                ]);

                try {
                    console.log("Unified Query: Calling GraphAPI.unifiedQuery...");
                    await GraphAPI.unifiedQuery(userQuery, currentModel, canvasId);
                    console.log("Unified Query: GraphAPI.unifiedQuery returned successfully.");
                } catch (e) {
                    console.error("Unified Query Invoke Error:", e);
                    // Remove the placeholder if invocation fails immediately
                    setMessages(prev => prev.slice(0, -1));
                }
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

            <div className="flex-1 overflow-y-auto custom-scrollbar p-4 flex flex-col">
                {messages.length === 0 ? (
                    <div className="flex-1 flex flex-col items-center justify-center text-center opacity-50">
                        <span className="material-icons-round text-4xl text-primary/40">smart_toy</span>
                        <p className="text-lg font-medium text-slate-400 dark:text-slate-500 mt-4">How can I help?</p>
                        <p className="text-sm text-slate-400 dark:text-slate-500">Ask questions across all your interconnected nodes.</p>
                    </div>
                ) : (
                    <div className="space-y-4">
                        {messages.map((msg, idx) => (
                            <div key={idx} className={`flex flex-col ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
                                <div className={`max-w-[90%] rounded-lg p-3 text-sm ${msg.role === 'user'
                                    ? 'bg-primary text-white'
                                    : 'bg-slate-100 dark:bg-slate-800 text-slate-800 dark:text-slate-200'
                                    }`}>
                                    <p className="whitespace-pre-wrap">{msg.content}</p>
                                </div>
                            </div>
                        ))}
                        <div ref={messagesEndRef} />
                    </div>
                )}
            </div>

            <div className="p-4 border-t border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark">
                <div className="relative group">
                    <div className="bg-slate-100 dark:bg-slate-900 rounded-lg p-3 border border-transparent transition-all shadow-sm">
                        <textarea
                            value={query}
                            onChange={(e) => setQuery(e.target.value)}
                            onKeyDown={handleKeyDown}
                            className="w-full bg-transparent border-none p-0 text-sm focus:ring-0 focus:outline-none resize-none h-12 placeholder-slate-400 dark:placeholder-slate-500 text-slate-800 dark:text-slate-200"
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
