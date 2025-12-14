
import {
    ReactFlow,
    Background,
    Controls,
    MiniMap,
    useNodesState,
    useEdgesState,
    Position,
    type Edge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { ChatNode, type ChatNodeType } from './ChatNode';

const nodeTypes = {
    chatNode: ChatNode,
};

// Initial Nodes Data
const initialNodes: ChatNodeType[] = [
    {
        id: '1',
        type: 'chatNode',
        position: { x: 100, y: 100 },
        data: {
            title: 'hello',
            content: (
                <div className="p-4 h-40 overflow-y-auto">
                    <p className="text-slate-600 dark:text-slate-300">hello, How are you?</p>
                </div>
            ),
            containerClassName: 'w-80',
            handles: {
                source: [Position.Right, Position.Bottom],
            }
        },
    },
    {
        id: '2',
        type: 'chatNode',
        position: { x: 500, y: 100 },
        data: {
            title: 'greeting',
            content: (
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
            ),
            containerClassName: 'w-80',
            handles: {
                target: [Position.Left],
                source: [Position.Top]
            }
        },
    },
];

const initialEdges: Edge[] = [
    { id: 'e1-2', source: '1', target: '2', animated: true },
];

export const CanvasBoard = () => {
    const [nodes, , onNodesChange] = useNodesState(initialNodes);
    const [edges, , onEdgesChange] = useEdgesState(initialEdges);

    return (
        <main className="flex-1 h-screen w-full bg-background-light dark:bg-background-dark">
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                nodeTypes={nodeTypes}

                defaultViewport={{ x: 0, y: 0, zoom: 0.85 }}
                className="bg-background-light dark:bg-background-dark"
            >
                <Background
                    className="bg-grid-pattern opacity-50"
                    gap={20}
                    size={1}
                    color="transparent" // Using our custom CSS background pattern instead of React Flow's SVGs for exact match
                />
                <Controls className="!bg-surface-light dark:!bg-surface-dark !border-border-light dark:!border-border-dark !shadow-lg !rounded-full !p-1.5 [&>button]:!border-none [&>button]:!rounded-full [&>button]:hover:!bg-slate-100 dark:[&>button]:hover:!bg-slate-800 [&>button]:!text-slate-500 dark:[&>button]:!text-slate-400 [&>button]:!transition-colors" />
                <MiniMap
                    className="!bg-surface-light dark:!bg-surface-dark !rounded-lg !shadow-lg !border !border-border-light dark:!border-border-dark overflow-hidden"
                    maskColor="rgba(241, 245, 249, 0.6)" // slate-100 with opacity
                    nodeColor={() => {
                        return '#e2e8f0'; // slate-200
                    }}
                />

                {/* Top Left Add Button Overlay */}
                <div className="absolute top-6 left-6 z-10 pointer-events-none">
                    <button className="pointer-events-auto w-10 h-10 bg-surface-light dark:bg-surface-dark rounded-full shadow-lg border border-border-light dark:border-border-dark flex items-center justify-center text-slate-600 dark:text-slate-300 hover:text-primary transition-colors">
                        <span className="material-icons-round text-2xl">add</span>
                    </button>
                </div>
            </ReactFlow>
        </main>
    );
};
