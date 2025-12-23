// Define the data structure for chat node (after submitting - displaying messages)
import { type ReactNode, useState } from 'react';
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';

export type ChatNodeData = {
    title?: string;
    content: string;
    footer?: ReactNode;
    headerClassName?: string;
    containerClassName?: string;
    onDelete?: (id: string) => void;
    onAddNode?: (direction: 'top' | 'bottom' | 'left' | 'right') => void;
    handles?: {
        source?: Position[];
        target?: Position[];
    };
};

// Define the generic node type
export type ChatNodeType = Node<ChatNodeData, 'chatNode'>;

export const ChatNode = ({ id, data }: NodeProps<ChatNodeType>) => {
    const {
        title = 'Chat',
        content,
        footer,
        headerClassName = "border-b border-border-light dark:border-border-dark",
        containerClassName = "",
        onDelete,
        onAddNode,
        handles = { source: [], target: [] }
    } = data;

    const [isMenuOpen, setIsMenuOpen] = useState(false);

    return (
        <div className={`bg-white dark:bg-surface-dark rounded-2xl shadow-sm border border-slate-200 dark:border-border-dark flex flex-col min-w-[320px] max-w-[400px] transition-shadow hover:shadow-md ${containerClassName} relative group`}>
            {/* Target Handles (Inputs) */}
            {handles.target?.map((pos) => (
                <Handle
                    key={`target-${pos}`}
                    type="target"
                    position={pos}
                    id={`target-${pos}`}
                    className="!w-3 !h-3 !bg-slate-300 dark:!bg-slate-500 !border-2 !border-white dark:!border-slate-800 opacity-0 group-hover:opacity-100 transition-opacity"
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
                                {/* Arrow pointing up */}
                                <div className="absolute -top-1.5 right-3 w-3 h-3 bg-white dark:bg-slate-800 transform rotate-45 border-t border-l border-slate-100 dark:border-slate-700"></div>

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
                                    <div className="h-px bg-slate-100 dark:bg-slate-700 my-1"></div>
                                    <button className="w-full text-left px-4 py-2.5 text-sm text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors">
                                        Edit
                                    </button>
                                    <button className="w-full text-left px-4 py-2.5 text-sm text-slate-600 dark:text-slate-300 hover:bg-slate-50 dark:hover:bg-slate-700 transition-colors">
                                        Duplicate
                                    </button>
                                </div>
                            </div>
                        </>
                    )}
                </div>
            </div>

            {/* Content: Display message */}
            <div className="p-4">
                <p className="text-sm text-slate-700 dark:text-slate-300 leading-relaxed whitespace-pre-wrap">
                    {content}
                </p>
            </div>

            {/* Optional Footer */}
            {footer && (
                <div className="px-4 py-3 text-xs text-slate-400 dark:text-slate-500 font-medium">
                    {footer}
                </div>
            )}

            {/* Source Handles (Outputs) */}
            {handles.source?.map((pos) => (
                <Handle
                    key={`source-${pos}`}
                    type="source"
                    position={pos}
                    id={`source-${pos}`}
                    className="!w-3 !h-3 !bg-slate-300 dark:!bg-slate-500 !border-2 !border-white dark:!border-slate-800 opacity-0 group-hover:opacity-100 transition-opacity"
                />
            ))}

            {/* Interactive Dots for adding new nodes */}
            <div className="absolute inset-0 pointer-events-none">
                <div
                    onClick={(e) => { e.stopPropagation(); onAddNode?.('top'); }}
                    className="absolute -top-1 left-1/2 -translate-x-1/2 w-2.5 h-2.5 bg-slate-400 dark:bg-slate-600 rounded-full opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:scale-125 transition-all cursor-pointer pointer-events-auto z-10"
                    title="Add node above"
                />
                <div
                    onClick={(e) => { e.stopPropagation(); onAddNode?.('bottom'); }}
                    className="absolute -bottom-1 left-1/2 -translate-x-1/2 w-2.5 h-2.5 bg-slate-400 dark:bg-slate-600 rounded-full opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:scale-125 transition-all cursor-pointer pointer-events-auto z-10"
                    title="Add node below"
                />
                <div
                    onClick={(e) => { e.stopPropagation(); onAddNode?.('left'); }}
                    className="absolute top-1/2 -left-1 -translate-y-1/2 w-2.5 h-2.5 bg-slate-400 dark:bg-slate-600 rounded-full opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:scale-125 transition-all cursor-pointer pointer-events-auto z-10"
                    title="Add node to the left"
                />
                <div
                    onClick={(e) => { e.stopPropagation(); onAddNode?.('right'); }}
                    className="absolute top-1/2 -right-1 -translate-y-1/2 w-2.5 h-2.5 bg-slate-400 dark:bg-slate-600 rounded-full opacity-0 group-hover:opacity-60 hover:!opacity-100 hover:scale-125 transition-all cursor-pointer pointer-events-auto z-10"
                    title="Add node to the right"
                />
            </div>
        </div>
    );
};
