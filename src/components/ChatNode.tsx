import { type ReactNode } from 'react';
import { Handle, Position, type NodeProps, type Node } from '@xyflow/react';

// Define the data structure for our custom node
export type ChatNodeData = {
    title?: ReactNode;
    content: ReactNode;
    headerClassName?: string;
    containerClassName?: string;
    hideHeader?: boolean;
    handles?: {
        source?: Position[];
        target?: Position[];
    };
};

// Define the generic node type
export type ChatNodeType = Node<ChatNodeData, 'chatNode'>;

export const ChatNode = ({ data }: NodeProps<ChatNodeType>) => {
    const {
        title,
        content,
        headerClassName = "border-b border-border-light dark:border-border-dark",
        containerClassName = "",
        hideHeader = false,
        handles = { source: [], target: [] }
    } = data;

    return (
        <div className={`bg-surface-light dark:bg-surface-dark rounded-xl shadow-lg border border-border-light dark:border-border-dark flex flex-col min-w-[300px] ${containerClassName}`}>
            {/* Target Handles (Inputs) */}
            {handles.target?.map((pos, index) => (
                <Handle
                    key={`target-${index}`}
                    type="target"
                    position={pos}
                    className="!w-3 !h-3 !bg-slate-400 dark:!bg-slate-500 !border-2 !border-white dark:!border-slate-800"
                />
            ))}

            {!hideHeader && (
                <div className={`p-3 flex items-center justify-between ${headerClassName}`}>
                    <div className="flex items-center gap-2">
                        {typeof title === 'string' ? <span className="font-medium text-sm text-slate-700 dark:text-slate-200">{title}</span> : title}
                    </div>
                    <span className="material-icons-round text-slate-400 text-sm">more_horiz</span>
                </div>
            )}

            {content}

            {/* Source Handles (Outputs) */}
            {handles.source?.map((pos, index) => (
                <Handle
                    key={`source-${index}`}
                    type="source"
                    position={pos}
                    className="!w-3 !h-3 !bg-slate-400 dark:!bg-slate-500 !border-2 !border-white dark:!border-slate-800"
                />
            ))}
        </div>
    );
};
