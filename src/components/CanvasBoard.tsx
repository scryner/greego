import React from 'react';
import { ChatNode, Handle } from './ChatNode';

export const CanvasBoard = () => {
    return (
        <main className="flex-1 relative overflow-hidden bg-grid-pattern h-screen">
            {/* Top Left Add Button */}
            <div className="absolute top-6 left-6 z-10">
                <button className="w-10 h-10 bg-surface-light dark:bg-surface-dark rounded-full shadow-lg border border-border-light dark:border-border-dark flex items-center justify-center text-slate-600 dark:text-slate-300 hover:text-primary transition-colors">
                    <span className="material-icons-round text-2xl">add</span>
                </button>
            </div>

            {/* SVG Background Layer */}
            <div className="absolute inset-0 w-[2000px] h-[2000px] transform origin-top-left pointer-events-none">
                <svg className="absolute inset-0 w-full h-full z-0">
                    <path className="dark:stroke-slate-600" d="M 400 143 C 460 143, 460 199, 520 199" fill="none" stroke="#cbd5e1" strokeWidth="2"></path>
                    <path className="dark:stroke-slate-600" d="M 776 199 C 808 199, 808 215, 840 215" fill="none" stroke="#cbd5e1" strokeWidth="2"></path>
                    <path className="dark:stroke-slate-600" d="M 240 245 L 240 320" fill="none" stroke="#cbd5e1" strokeDasharray="6,4" strokeWidth="2"></path>
                </svg>
            </div>

            {/* Nodes */}

            {/* Node 1: Hello */}
            <ChatNode
                title="hello"
                containerClassName="top-10 left-20 w-80"
                handles={<><Handle position="right" /><Handle position="bottom" /></>}
            >
                <div className="p-4 h-40 overflow-y-auto">
                    <p className="text-slate-600 dark:text-slate-300">hello, How are you?</p>
                </div>
            </ChatNode>

            {/* Node 2: Greeting (Input) */}
            <ChatNode
                title="greeting"
                containerClassName="top-80 left-20 w-80"
                handles={<Handle position="top" />}
            >
                <div className="p-4 h-32 flex flex-col justify-end">
                    <div className="flex flex-col gap-3">
                        <div className="relative w-full">
                            <div className="absolute left-2 top-1/2 -translate-y-1/2 flex items-center justify-center w-5 h-5 bg-slate-500 rounded-full text-white">
                                <span className="material-icons-round text-[14px]">add</span>
                            </div>
                            <input className="w-full text-sm pl-9 pr-4 py-2 rounded-full bg-slate-200/50 dark:bg-slate-800 border-none focus:ring-0 placeholder-slate-500 text-slate-700 dark:text-slate-200 transition-colors" placeholder="Ask anything..." type="text" />
                        </div>
                        <div className="flex items-center gap-2 text-slate-400 dark:text-slate-500 px-1">
                            <span className="text-xs font-medium">lms/gpt-oss-120b</span>
                            <div className="flex-1"></div>
                        </div>
                    </div>
                </div>
            </ChatNode>

            {/* Node 3: Translating */}
            <ChatNode
                title={<><span className="material-icons-round text-indigo-500 text-sm">translate</span><span className="font-medium text-sm text-indigo-700 dark:text-indigo-300">translating</span></>}
                containerClassName="top-20 left-[520px] w-64"
                headerClassName="p-3 border-b border-border-light dark:border-border-dark flex items-center justify-between bg-indigo-50 dark:bg-indigo-900/20 rounded-t-xl"
                handles={<><Handle position="left" /><Handle position="right" /></>}
            >
                <div className="p-4 h-48 overflow-y-auto">
                    <div className="text-sm font-medium text-slate-500 dark:text-slate-400 mb-2">Korean Output:</div>
                    <p className="text-slate-800 dark:text-slate-200 leading-relaxed font-sans">
                        한국어로는<br />
                        &quot;안녕, 오늘 어때?&quot;
                    </p>
                </div>
            </ChatNode>

            {/* Node 4: Another Prompt Input (Headless) */}
            <ChatNode
                hideHeader
                containerClassName="top-[167px] left-[840px] w-72"
                handles={<Handle position="left" />}
            >
                <div className="p-4">
                    <div className="flex flex-col gap-3">
                        <div className="relative w-full">
                            <div className="absolute left-2 top-1/2 -translate-y-1/2 flex items-center justify-center w-5 h-5 bg-slate-500 rounded-full text-white">
                                <span className="material-icons-round text-[14px]">add</span>
                            </div>
                            <input className="w-full text-sm pl-9 pr-4 py-2 rounded-full bg-slate-200/50 dark:bg-slate-800 border-none focus:ring-0 placeholder-slate-500 text-slate-700 dark:text-slate-200 transition-colors" placeholder="Ask anything..." type="text" />
                        </div>
                        <div className="flex items-center gap-2 text-slate-400 dark:text-slate-500 px-1">
                            <span className="text-xs font-medium">lms/gpt-oss-120b</span>
                            <div className="flex-1"></div>
                        </div>
                    </div>
                </div>
            </ChatNode>

            {/* MiniMap */}
            <div className="absolute bottom-6 left-6 w-32 h-24 bg-surface-light dark:bg-surface-dark rounded-lg shadow-lg border border-border-light dark:border-border-dark overflow-hidden z-20">
                <div className="w-full h-full relative bg-slate-50/80 dark:bg-slate-900/80 backdrop-blur-sm">
                    <svg className="absolute inset-0 w-full h-full pointer-events-none z-0 opacity-50">
                        <path className="text-slate-400 dark:text-slate-500" d="M 40 18 C 47 18, 47 24, 54 24" fill="none" stroke="currentColor" strokeWidth="1.5"></path>
                        <path className="text-slate-400 dark:text-slate-500" d="M 78 24 C 82 24, 82 26, 86 26" fill="none" stroke="currentColor" strokeWidth="1.5"></path>
                        <path className="text-slate-400 dark:text-slate-500" d="M 24 28 L 24 38" fill="none" stroke="currentColor" strokeDasharray="2,2" strokeWidth="1.5"></path>
                    </svg>
                    <div className="absolute top-2 left-2 w-8 h-5 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded-[2px] shadow-sm flex flex-col overflow-hidden z-10">
                        <div className="h-1 bg-slate-200 dark:bg-slate-700 w-full"></div>
                    </div>
                    <div className="absolute top-10 left-2 w-8 h-4 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded-[2px] shadow-sm flex flex-col overflow-hidden z-10">
                        <div className="h-1 bg-slate-200 dark:bg-slate-700 w-full"></div>
                    </div>
                    <div className="absolute top-3 left-[54px] w-6 h-6 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded-[2px] shadow-sm flex flex-col overflow-hidden z-10">
                        <div className="h-1 bg-indigo-200 dark:bg-indigo-800 w-full"></div>
                    </div>
                    <div className="absolute top-5 left-[86px] w-7 h-3 bg-white dark:bg-slate-800 border border-slate-300 dark:border-slate-600 rounded-[2px] shadow-sm flex flex-col justify-center px-[2px] z-10">
                        <div className="w-full h-1 bg-slate-100 dark:bg-slate-600 rounded-full"></div>
                    </div>
                    <div className="absolute top-0 left-0 w-24 h-16 border-2 border-primary rounded-sm pointer-events-none z-20 opacity-50 bg-primary/5"></div>
                </div>
            </div>

            {/* Controls */}
            <div className="absolute bottom-6 left-1/2 transform -translate-x-1/2 z-20">
                <div className="flex items-center gap-1 bg-surface-light dark:bg-surface-dark rounded-full shadow-lg border border-border-light dark:border-border-dark p-1.5">
                    <button className="p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-500 dark:text-slate-400 transition-colors">
                        <span className="material-icons-round text-xl">remove</span>
                    </button>
                    <button className="p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-500 dark:text-slate-400 transition-colors">
                        <span className="material-icons-round text-xl">add</span>
                    </button>
                    <div className="w-px h-6 bg-border-light dark:bg-border-dark mx-1"></div>
                    <button className="p-2 rounded-full hover:bg-slate-100 dark:hover:bg-slate-800 text-slate-500 dark:text-slate-400 transition-colors" title="Fit to screen">
                        <span className="material-icons-round text-xl">center_focus_strong</span>
                    </button>
                </div>
            </div>
        </main>
    );
};
