
import {
    ReactFlow,
    Background,
    Controls,
    MiniMap,
    useNodesState,
    useEdgesState,
    type Edge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { ChatNode, type ChatNodeType } from './ChatNode';

const nodeTypes = {
    chatNode: ChatNode,
};

// Initial Nodes Data
const initialNodes: ChatNodeType[] = [];

const initialEdges: Edge[] = [];

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
