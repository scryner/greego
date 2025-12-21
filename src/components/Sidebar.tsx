

export const Sidebar = () => {
    return (
        <aside className="w-80 flex flex-col border-r border-border-light dark:border-border-dark bg-surface-light dark:bg-surface-dark shadow-sm z-20 flex-shrink-0">
            <div className="h-12 px-4 flex items-center justify-between border-b border-border-light dark:border-border-dark">
                <button className="p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-500 dark:text-slate-400 transition-colors">
                    <span className="material-icons-round">menu</span>
                </button>
                <h1 className="text-base font-semibold text-slate-800 dark:text-white">Subject</h1>
                <div className="flex items-center gap-2">
                    <button className="w-8 h-8 flex items-center justify-center rounded-lg hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-400 dark:text-slate-500 transition-colors">
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
                        <textarea className="w-full bg-transparent border-none p-0 text-sm focus:ring-0 resize-none h-12 placeholder-slate-400 dark:placeholder-slate-500 text-slate-800 dark:text-slate-200" placeholder="Ask anything..."></textarea>
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
                    <div className="mt-2 flex items-center gap-2 text-slate-400 dark:text-slate-500 px-1">
                        <span className="text-xs font-medium">lms/gpt-oss-120b</span>
                    </div>
                </div>
            </div>
        </aside>
    );
};
