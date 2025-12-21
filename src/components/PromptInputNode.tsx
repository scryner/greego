// Define the data structure for prompt input node (before submitting)
import { type ReactNode, useState, useEffect } from 'react';
import { Handle, Position, type NodeProps, type Node, useHandleConnections, useNodesData } from '@xyflow/react';
import { invoke } from '@tauri-apps/api/core';

export type PromptInputNodeData = {
    title?: string;
    parentId?: string;
    relationType?: 'sequence' | 'derive';
    selectedModel?: string; // Add selectedModel
    onModelSelect?: (model: string) => void; // Add callback
    footer?: ReactNode;
    headerClassName?: string;
    containerClassName?: string;
    onDelete?: (id: string) => void;
    onSubmit?: (text: string, nodeId: string) => void;
    handles?: {
        source?: Position[];
        target?: Position[];
    };
};

// Define the generic node type
export type PromptInputNodeType = Node<PromptInputNodeData, 'promptInputNode'>;

export const PromptInputNode = ({ id, data }: NodeProps<PromptInputNodeType>) => {
    const {
        title = 'New chat',
        headerClassName = "border-b border-border-light dark:border-border-dark",
        containerClassName = "",
        onDelete,
        onSubmit,
        handles = { source: [], target: [] }
    } = data;

    const [isMenuOpen, setIsMenuOpen] = useState(false);

    // Model selection state
    const [selectedModel, setSelectedModel] = useState<string>("");
    const [availableModels, setAvailableModels] = useState<string[]>([]);
    const [isModelSelectorOpen, setIsModelSelectorOpen] = useState(false);

    // React Flow hooks to inspect upstream connection
    const connections = useHandleConnections({
        type: 'target',
    });
    // We assume the first connection is the primary flow
    const upstreamNodeData = useNodesData(connections[0]?.source) as any;

    // Effect: Inherit model from upstream node on mount or connection change
    useEffect(() => {
        if (connections.length > 0 && upstreamNodeData?.selectedModel) {
            setSelectedModel(upstreamNodeData.selectedModel);
        } else {
            // No upstream connection or no model in upstream -> Default empty
            setSelectedModel("");
        }
    }, [connections.length, upstreamNodeData]);

    const handleModelSelectorClick = async () => {
        if (!isModelSelectorOpen) {
            try {
                const models = await invoke<string[]>('get_llm_available_models');
                setAvailableModels(models);
            } catch (error) {
                console.error("Failed to fetch models:", error);
            }
        }
        setIsModelSelectorOpen(!isModelSelectorOpen);
    };

    return (
        <div className={`bg-white dark:bg-surface-dark rounded-2xl shadow-sm border border-slate-200 dark:border-border-dark flex flex-col min-w-[320px] max-w-[400px] transition-shadow hover:shadow-md ${containerClassName} relative group`}>
            {/* Target Handles (Inputs) */}
            {handles.target?.map((pos) => (
                <Handle
                    key={`target-${pos}`}
                    type="target"
                    position={pos}
                    id={`target-${pos}`}
                    className="!w-3 !h-3 !bg-slate-300 dark:!bg-slate-500 !border-2 !border-white dark:!border-slate-800"
                />
            ))}

            {/* Header */}
            <div className={`px-4 py-1.5 flex items-center justify-between ${headerClassName}`}>
                <div className="flex items-center gap-2">
                    <span className="font-semibold text-slate-800 dark:text-slate-200 text-base">{title}</span>
                </div>

                {/* Ellipsis menu container */}
                <div className="relative">
                    <button
                        onClick={() => setIsMenuOpen(!isMenuOpen)}
                        className="text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 transition-colors p-1 rounded-md hover:bg-slate-100 dark:hover:bg-slate-800 focus:outline-none">
                        <span className="material-icons-round text-xl">more_horiz</span>
                    </button>

                    {/* Popup Menu */}
                    {isMenuOpen && (
                        <>
                            {/* Backdrop to close menu on click outside */}
                            <div
                                className="fixed inset-0 z-40"
                                onClick={() => setIsMenuOpen(false)}
                            />

                            <div className="absolute right-0 top-full mt-2 w-40 bg-white dark:bg-slate-800 rounded-lg shadow-xl border border-slate-100 dark:border-slate-700 z-50 overflow-hidden animate-in fade-in zoom-in-95 duration-100 origin-top-right">
                                <div className="flex flex-col py-1 relative bg-white dark:bg-slate-800 rounded-lg">
                                    <button
                                        onClick={() => {
                                            if (onDelete) onDelete(id);
                                            setIsMenuOpen(false);
                                        }}
                                        className="w-full text-left px-4 py-2.5 text-sm text-red-600 dark:text-red-400 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors flex items-center gap-2"
                                    >
                                        <span className="material-icons-round text-lg">delete_outline</span>
                                        Delete
                                    </button>
                                </div>
                            </div>
                        </>
                    )}
                </div>
            </div>



            {/* Content: Input Field */}
            <div className="p-4">
                <div className="w-full">
                    {/* Pill-shaped input container */}
                    <div className="relative group">
                        <input
                            type="text"
                            className="w-full pl-10 pr-4 py-3 bg-slate-100 dark:bg-slate-800 rounded-full text-sm text-slate-700 dark:text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-medium"
                            placeholder="Ask anything..."
                            autoFocus
                            onKeyDown={(e) => {
                                if (e.key === 'Enter' && !e.nativeEvent.isComposing) {
                                    e.preventDefault();
                                    if (onSubmit) {
                                        // Pass selected model? onSubmit signature is (text, nodeId).
                                        // Usually data update happens via hooks or separate context.
                                        // We might want to pass the model to the node data so it executes with it?
                                        // For now, adhering to existing onSubmit signature.
                                        onSubmit(e.currentTarget.value, id);
                                    }
                                }
                            }}
                        />
                        {/* Plus icon inside the input */}
                        <div className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 bg-slate-500 dark:bg-slate-600 rounded-full flex items-center justify-center text-white pointer-events-none">
                            <span className="material-icons-round text-sm">add</span>
                        </div>
                    </div>
                </div>
            </div>

            {/* Footer with Model Selector */}
            <div className="px-4 py-3 text-xs text-slate-400 dark:text-slate-500 font-medium border-t border-slate-100 dark:border-slate-700/50">
                <div className="relative inline-block">
                    <button
                        onClick={handleModelSelectorClick}
                        className="flex items-center gap-1 hover:text-slate-600 dark:hover:text-slate-300 transition-colors"
                    >
                        {selectedModel || "Select model"}
                        <span className="material-icons-round text-[10px] opacity-70">expand_more</span>
                    </button>

                    {isModelSelectorOpen && (
                        <>
                            {/* Transparent backdrop for model menu */}
                            <div
                                className="fixed inset-0 z-40 cursor-default"
                                onClick={() => setIsModelSelectorOpen(false)}
                            />
                            <div className="absolute left-0 top-full mt-1 w-56 bg-white dark:bg-slate-800 rounded-lg shadow-xl border border-slate-200 dark:border-slate-700 z-50 overflow-hidden max-h-60 overflow-y-auto">
                                {availableModels.length > 0 ? (
                                    availableModels.map((modelStr) => (
                                        <button
                                            key={modelStr}
                                            onClick={() => {
                                                setSelectedModel(modelStr);
                                                setIsModelSelectorOpen(false);
                                                if (data.onModelSelect) {
                                                    data.onModelSelect(modelStr);
                                                }
                                            }}
                                            className={`w-full text-left px-3 py-2 text-xs transition-colors truncate ${selectedModel === modelStr ? 'bg-primary/10 text-primary font-semibold' : 'text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700'}`}
                                        >
                                            {modelStr}
                                        </button>
                                    ))
                                ) : (
                                    <div className="px-3 py-2 text-xs text-slate-400 italic">No models found</div>
                                )}
                            </div>
                        </>
                    )}
                </div>
            </div>

            {/* Source Handles (Outputs) */}
            {handles.source?.map((pos) => (
                <Handle
                    key={`source-${pos}`}
                    type="source"
                    position={pos}
                    id={`source-${pos}`}
                    className="!w-3 !h-3 !bg-slate-300 dark:!bg-slate-500 !border-2 !border-white dark:!border-slate-800"
                />
            ))}
        </div>
    );
};
