
import React from 'react';

interface SettingsWindowProps {
    onClose: () => void;
}

export const SettingsWindow: React.FC<SettingsWindowProps> = ({ onClose }) => {
    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-12 bg-black/50 backdrop-blur-sm" onClick={(e) => {
            if (e.target === e.currentTarget) onClose();
        }}>
            <div className="bg-white dark:bg-card-dark w-full max-w-[1200px] h-[800px] rounded-3xl shadow-2xl flex overflow-hidden border border-border-light dark:border-border-dark animate-in fade-in zoom-in duration-200">
                {/* Sidebar */}
                <div className="w-[280px] bg-background-light dark:bg-background-dark border-r border-border-light dark:border-border-dark flex flex-col p-4 space-y-6 overflow-y-auto">
                    {/* Search */}
                    <div className="relative">
                        <span className="absolute inset-y-0 left-3 flex items-center text-slate-400">
                            <span className="material-icons-round text-lg">search</span>
                        </span>
                        <input
                            type="text"
                            placeholder="Search Settings"
                            className="w-full bg-slate-200 dark:bg-slate-800 rounded-lg py-2 pl-10 pr-4 text-sm outline-none focus:ring-2 focus:ring-primary/50 text-slate-700 dark:text-slate-200 placeholder-slate-500"
                        />
                    </div>

                    {/* Menu Items */}
                    <nav className="space-y-6">
                        <div>
                            <h3 className="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-2 px-3">AI</h3>
                            <button className="w-full flex items-center gap-3 px-3 py-2 rounded-lg bg-primary text-white text-left text-sm font-medium shadow-md shadow-primary/20">
                                <span className="material-icons-round text-lg">auto_awesome</span>
                                LLM services
                            </button>
                            <button className="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-slate-600 dark:text-slate-400 hover:bg-slate-200 dark:hover:bg-slate-800 transition-colors text-left text-sm font-medium">
                                <span className="material-icons-round text-lg">format_list_numbered</span>
                                Prompts
                            </button>
                        </div>
                    </nav>
                </div>

                {/* Main Content */}
                <div className="flex-1 bg-white dark:bg-card-dark overflow-y-auto p-8 relative">
                    <button onClick={onClose} className="absolute top-4 right-4 p-2 rounded-lg hover:bg-slate-100 dark:hover:bg-slate-700 text-slate-400 transition-colors">
                        <span className="material-icons-round">close</span>
                    </button>

                    <h2 className="text-xl font-bold mb-6 text-slate-800 dark:text-slate-100">LLM services</h2>

                    {/* Active Services List */}
                    <div className="space-y-4 mb-12">
                        {/* Foundation Model */}
                        <div className="bg-slate-50 dark:bg-slate-900 rounded-xl p-4 flex items-center justify-between">
                            <div className="flex items-center gap-4">
                                <div className="w-10 h-10 rounded-lg bg-black text-white flex items-center justify-center">
                                    <span className="material-icons-round text-2xl">apple</span>
                                </div>
                                <div>
                                    <h4 className="font-bold text-slate-800 dark:text-slate-200">Foundation Model</h4>
                                    <p className="text-xs text-slate-500">Apple</p>
                                </div>
                            </div>
                        </div>

                        {/* OpenAI */}
                        <div className="bg-slate-50 dark:bg-slate-900 rounded-xl p-4 flex items-center justify-between hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
                            <div className="flex items-center gap-4">
                                <div className="w-10 h-10 rounded-lg bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 flex items-center justify-center">
                                    <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 24 24"><path d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.0462 6.0462 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0843 5.46-3.1511 1.665 3.1568c-.1187.02-.2375.04-.3612.04a4.472 4.472 0 0 1-4.0293-2.6177zm6.2481-5.5812-1.9925-3.7506 2.6415-1.5247a4.3843 4.3843 0 0 1 .4381 3.6616 4.4839 4.4839 0 0 1-1.0871 1.6137zM2.341 8.0581a4.4698 4.4698 0 0 1 .5466-1.9213 4.4626 4.4626 0 0 1 3.531-2.3521l-1.6603 3.1568-2.4173 1.1166zm15.2222-2.8707-5.466 3.1511-1.665-3.1568c.1188-.02.2375-.04.3612-.04a4.472 4.472 0 0 1 4.0341 2.6177L16.2737 4.5l1.2895.6874zm-2.52 5.0666-6.308-3.6366 2.8715-1.6579 6.308 3.6366-2.8715 1.6579zM6.92 12.052l6.308 3.6366-2.8715 1.6579-6.308-3.6366 2.8715-1.6579zm-.2135 6.4197-2.6415 1.5247a4.3843 4.3843 0 0 1-.4381-3.6616 4.4839 4.4839 0 0 1 1.0871-1.6137l1.9925 3.7506zm13.9922-5.6577-2.4173-1.1166 1.6603-3.1568a4.4698 4.4698 0 0 1 2.9855 4.2734zm-9.0173 4.5269 1.665 3.1568c-2.4842.2706-4.9082-.8715-6.1772-2.9355l1.646-3.3732 2.8662 3.1519z"></path></svg>
                                </div>
                                <div>
                                    <h4 className="font-bold text-slate-800 dark:text-slate-200">OpenAI</h4>
                                    <p className="text-xs text-slate-500">Add API key to setup</p>
                                </div>
                            </div>
                            <button className="px-4 py-1.5 rounded-full bg-slate-400/50 hover:bg-slate-400 text-white text-sm font-medium transition-colors">
                                Configure
                            </button>
                        </div>

                        {/* Gemini */}
                        <div className="bg-slate-50 dark:bg-slate-900 rounded-xl p-4 flex items-center justify-between hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
                            <div className="flex items-center gap-4">
                                <div className="w-10 h-10 rounded-lg bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 flex items-center justify-center">
                                    <span className="material-icons-round">auto_awesome</span>
                                </div>
                                <div>
                                    <h4 className="font-bold text-slate-800 dark:text-slate-200">gemini-2.5-flash</h4>
                                    <p className="text-xs text-slate-500">Configure to use</p>
                                </div>
                            </div>
                            <button className="px-4 py-1.5 rounded-full bg-slate-400/50 hover:bg-slate-400 text-white text-sm font-medium transition-colors">
                                Configure
                            </button>
                        </div>

                        {/* Ollama */}
                        <div className="bg-slate-50 dark:bg-slate-900 rounded-xl p-4 flex items-center justify-between hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
                            <div className="flex items-center gap-4">
                                <div className="w-10 h-10 rounded-lg bg-white dark:bg-slate-800 border border-slate-200 dark:border-slate-700 flex items-center justify-center">
                                    <span className="material-icons-round text-slate-700 dark:text-slate-300">smart_toy</span>
                                </div>
                                <div>
                                    <h4 className="font-bold text-slate-800 dark:text-slate-200">qwen3:30b-a3b</h4>
                                    <p className="text-xs text-slate-500">Ollama</p>
                                </div>
                            </div>
                            <button className="px-4 py-1.5 rounded-full bg-slate-400/50 hover:bg-slate-400 text-white text-sm font-medium transition-colors">
                                Configure
                            </button>
                        </div>

                        {/* LMStudio */}
                        <div className="bg-slate-50 dark:bg-slate-900 rounded-xl p-4 flex items-center justify-between hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
                            <div className="flex items-center gap-4">
                                <div className="w-10 h-10 rounded-lg bg-indigo-100 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-400 flex items-center justify-center">
                                    <span className="material-icons-round">dns</span>
                                </div>
                                <div>
                                    <h4 className="font-bold text-slate-800 dark:text-slate-200">openai/gpt-oss-120b</h4>
                                    <p className="text-xs text-slate-500">LMStudio</p>
                                </div>
                            </div>
                            <button className="px-4 py-1.5 rounded-full bg-primary text-white text-sm font-medium hover:bg-primary/90 transition-colors shadow-sm shadow-primary/30">
                                Configure
                            </button>
                        </div>

                        {/* Custom */}
                        <div className="bg-slate-50 dark:bg-slate-900 rounded-xl p-4 flex items-center justify-between hover:bg-slate-100 dark:hover:bg-slate-800 transition-colors">
                            <div className="flex items-center gap-4">
                                <div className="w-10 h-10 rounded-lg bg-slate-100 dark:bg-slate-800 flex items-center justify-center">
                                    <span className="material-icons-round text-slate-500">auto_fix_high</span>
                                </div>
                                <div>
                                    <h4 className="font-bold text-slate-800 dark:text-slate-200">Custom</h4>
                                    <p className="text-xs text-slate-500">Configure to use</p>
                                </div>
                            </div>
                            <button className="px-4 py-1.5 rounded-full bg-slate-400/50 hover:bg-slate-400 text-white text-sm font-medium transition-colors">
                                Configure
                            </button>
                        </div>
                    </div>

                    {/* Add another service */}
                    <h3 className="text-sm font-bold text-slate-800 dark:text-slate-100 mb-4">Add another service</h3>
                    <div className="grid grid-cols-4 gap-3">
                        {/* Google */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800 border border-transparent hover:border-slate-200 dark:hover:border-slate-700 transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-white dark:bg-slate-800 shadow-sm flex items-center justify-center text-blue-500">
                                    <span className="material-icons-round text-sm">auto_awesome</span>
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Google</span>
                            </div>
                            <span className="material-icons-round text-slate-300 group-hover:text-primary text-sm">add</span>
                        </button>

                        {/* LMStudio */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-indigo-50 dark:bg-indigo-900/20 hover:bg-indigo-100 dark:hover:bg-indigo-900/40 border border-transparent transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-indigo-600 text-white shadow-sm flex items-center justify-center">
                                    <span className="material-icons-round text-sm">dns</span>
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">LMStudio</span>
                            </div>
                            <span className="material-icons-round text-indigo-300 group-hover:text-indigo-500 text-sm">add</span>
                        </button>

                        {/* Ollama */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800 border border-transparent hover:border-slate-200 dark:hover:border-slate-700 transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-white dark:bg-slate-800 shadow-sm border border-slate-200 dark:border-slate-700 flex items-center justify-center">
                                    <span className="material-icons-round text-xs text-slate-700 dark:text-slate-300">smart_toy</span>
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Ollama</span>
                            </div>
                            <span className="material-icons-round text-slate-300 group-hover:text-primary text-sm">add</span>
                        </button>

                        {/* OpenAI */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-green-50 dark:bg-green-900/20 hover:bg-green-100 dark:hover:bg-green-900/40 border border-transparent transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-green-600 text-white shadow-sm flex items-center justify-center">
                                    <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 24 24"><path d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.0462 6.0462 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0843 5.46-3.1511 1.665 3.1568c-.1187.02-.2375.04-.3612.04a4.472 4.472 0 0 1-4.0293-2.6177zm6.2481-5.5812-1.9925-3.7506 2.6415-1.5247a4.3843 4.3843 0 0 1 .4381 3.6616 4.4839 4.4839 0 0 1-1.0871 1.6137zM2.341 8.0581a4.4698 4.4698 0 0 1 .5466-1.9213 4.4626 4.4626 0 0 1 3.531-2.3521l-1.6603 3.1568-2.4173 1.1166zm15.2222-2.8707-5.466 3.1511-1.665-3.1568c.1188-.02.2375-.04.3612-.04a4.472 4.472 0 0 1 4.0341 2.6177L16.2737 4.5l1.2895.6874zm-2.52 5.0666-6.308-3.6366 2.8715-1.6579 6.308 3.6366-2.8715 1.6579zM6.92 12.052l6.308 3.6366-2.8715 1.6579-6.308-3.6366 2.8715-1.6579zm-.2135 6.4197-2.6415 1.5247a4.3843 4.3843 0 0 1-.4381-3.6616 4.4839 4.4839 0 0 1 1.0871-1.6137l1.9925 3.7506zm13.9922-5.6577-2.4173-1.1166 1.6603-3.1568a4.4698 4.4698 0 0 1 2.9855 4.2734zm-9.0173 4.5269 1.665 3.1568c-2.4842.2706-4.9082-.8715-6.1772-2.9355l1.646-3.3732 2.8662 3.1519z"></path></svg>
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">OpenAI</span>
                            </div>
                            <span className="material-icons-round text-green-400 group-hover:text-green-600 text-sm">add</span>
                        </button>

                        {/* OpenRouter */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800 border border-transparent hover:border-slate-200 dark:hover:border-slate-700 transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-slate-500 text-white shadow-sm flex items-center justify-center">
                                    <span className="material-icons-round text-sm">router</span>
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">OpenRouter</span>
                            </div>
                            <span className="material-icons-round text-slate-300 group-hover:text-primary text-sm">add</span>
                        </button>

                        {/* Anthropic */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-orange-50 dark:bg-orange-900/20 hover:bg-orange-100 dark:hover:bg-orange-900/40 border border-transparent transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-orange-800 text-white shadow-sm flex items-center justify-center font-serif font-bold text-xs">
                                    AI
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Anthropic</span>
                            </div>
                            <span className="material-icons-round text-orange-300 group-hover:text-orange-500 text-sm">add</span>
                        </button>

                        {/* Azure */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-sky-50 dark:bg-sky-900/20 hover:bg-sky-100 dark:hover:bg-sky-900/40 border border-transparent transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-sky-600 text-white shadow-sm flex items-center justify-center">
                                    <span className="material-icons-round text-sm">cloud</span>
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Azure</span>
                            </div>
                            <span className="material-icons-round text-sky-300 group-hover:text-sky-500 text-sm">add</span>
                        </button>

                        {/* xAI */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-slate-100 dark:bg-slate-800 hover:bg-slate-200 dark:hover:bg-slate-700 border border-transparent transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-black text-white shadow-sm flex items-center justify-center font-bold text-xs">
                                    X
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">xAI</span>
                            </div>
                            <span className="material-icons-round text-slate-400 group-hover:text-slate-600 text-sm">add</span>
                        </button>

                        {/* Hugging Face */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-yellow-50 dark:bg-yellow-900/20 hover:bg-yellow-100 dark:hover:bg-yellow-900/40 border border-transparent transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-yellow-400 text-white shadow-sm flex items-center justify-center">
                                    <span className="text-xs">HF</span>
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Hugging Face</span>
                            </div>
                            <span className="material-icons-round text-yellow-300 group-hover:text-yellow-500 text-sm">add</span>
                        </button>

                        {/* Custom */}
                        <button className="flex items-center justify-between p-2 rounded-lg bg-slate-50 dark:bg-slate-900 hover:bg-slate-100 dark:hover:bg-slate-800 border border-transparent hover:border-slate-200 dark:hover:border-slate-700 transition-all group">
                            <div className="flex items-center gap-2">
                                <div className="w-6 h-6 rounded-md bg-slate-100 dark:bg-slate-700 flex items-center justify-center">
                                    <span className="material-icons-round text-xs text-slate-500">auto_fix_high</span>
                                </div>
                                <span className="text-sm font-medium text-slate-700 dark:text-slate-300">Custom</span>
                            </div>
                            <span className="material-icons-round text-slate-300 group-hover:text-primary text-sm">add</span>
                        </button>
                    </div>

                </div>
            </div>
        </div>
    );
};
