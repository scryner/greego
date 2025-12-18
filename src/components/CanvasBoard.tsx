
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
import { ModelSelector } from './ModelSelector';
import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { GraphAPI } from '../services/backend';

const nodeTypes = {
    chatNode: ChatNode,
};

// Initial Nodes Data
const initialNodes: ChatNodeType[] = [];

const initialEdges: Edge[] = [];

export const CanvasBoard = () => {
    const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        // Listen for backend errors
        const unlisten = listen<string>('db-error', (event) => {
            setError(event.payload);
            setTimeout(() => setError(null), 5000); // Clear after 5s
        });

        return () => {
            unlisten.then(u => u());
        };
    }, []);

    const handleDeleteNode = async (nodeId: string) => {
        if (nodeId.startsWith('temp-')) {
            // Just remove from state if it's a temp node
            setNodes((nds) => nds.filter((n) => n.id !== nodeId));
            return;
        }

        try {
            await GraphAPI.deleteNode(nodeId);
            setNodes((nds) => nds.filter((n) => n.id !== nodeId));
            // Also clean up connected edges
            setEdges((eds) => eds.filter(e => e.source !== nodeId && e.target !== nodeId));
        } catch (e) {
            console.error("Failed to delete node", e);
        }
    };

    const handleChatSubmit = async (text: string, tempNodeId: string) => {
        try {
            // Invoke backend
            // TODO: Use actual canvas ID. For now hardcoded or passed from somewhere? 
            // The prompt didn't specify multiple canvases, so "main" or similar is fine.
            const canvasId = "canvas:main";
            const newNodes = await GraphAPI.invokeChat(canvasId, text);

            // Transform backend nodes to ReactFlow nodes
            const rfNodes = newNodes.map(n => ({
                id: n.id as string,
                position: n.position,
                data: {
                    content: (n.type as any)?.data?.value?.text || JSON.stringify(n.data),
                    title: (n.type as any)?.data?.value?.role === 'user' ? 'Me' : 'AI',
                    onDelete: handleDeleteNode // Pass delete handler
                },
                type: 'chatNode'
            }));

            // Replace temp node with real nodes
            setNodes((nds) => {
                const filtered = nds.filter(n => n.id !== tempNodeId);
                return [...filtered, ...rfNodes as any];
            });

            // TODO: If we want edges, we need backend to return them or we link them locally?
            // The invoke_chat_command returns nodes. It also creates relation in DB.
            // But frontend needs edges to visualize connection.
            // For now, let's just show nodes. Ideally verify edges too.
            // Requirement says "connected...".
            // If backend returns only nodes, we might need to fetch edges or reload graph.
            // Optimistic approach: backend returns [UserNode, BotNode]. They are implicitly connected.
            // We can create an edge between them locally.
            if (newNodes.length >= 2) {
                const source = newNodes[0].id as string;
                const target = newNodes[1].id as string;
                const newEdge: Edge = {
                    id: `e-${source}-${target}`,
                    source,
                    target,
                };
                setEdges((eds) => [...eds, newEdge]);
            }

        } catch (e) {
            console.error("Failed to invoke chat", e);
            // Error listener will likely catch the emitted error too.
        }
    };

    const handleAddNode = () => {
        const id = `temp-${Date.now()}`;
        const newNode: ChatNodeType = {
            id,
            position: { x: 100, y: 100 }, // Initial position
            type: 'chatNode',
            data: {
                title: 'New chat', // Matches user request
                footer: (
                    <ModelSelector
                        currentModel="lms/gpt-oss-120b"
                        onModelSelect={(model) => console.log("Selected model:", model)}
                    />
                ),
                onDelete: handleDeleteNode, // Pass delete handler
                content: (
                    <div className="w-full">
                        {/* Pill-shaped input container */}
                        <div className="relative group">
                            <input
                                type="text"
                                className="w-full pl-10 pr-4 py-3 bg-slate-100 dark:bg-slate-800 rounded-full text-sm text-slate-700 dark:text-slate-200 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-primary/20 transition-all font-medium"
                                placeholder="Ask anything..."
                                autoFocus
                                onKeyDown={(e) => {
                                    if (e.key === 'Enter') {
                                        e.preventDefault();
                                        handleChatSubmit(e.currentTarget.value, id);
                                    }
                                }}
                            />
                            {/* Plus icon inside the input */}
                            <div className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 bg-slate-500 dark:bg-slate-600 rounded-full flex items-center justify-center text-white pointer-events-none">
                                <span className="material-icons-round text-sm">add</span>
                            </div>
                        </div>
                    </div>
                ),
            },
        };
        setNodes((nds) => [...nds, newNode]);
    };

    return (
        <main className="flex-1 h-screen w-full bg-background-light dark:bg-background-dark relative">
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
                    color="transparent"
                />
                <Controls className="!bg-surface-light dark:!bg-surface-dark !border-border-light dark:!border-border-dark !shadow-lg !rounded-full !p-1.5 [&>button]:!border-none [&>button]:!rounded-full [&>button]:hover:!bg-slate-100 dark:[&>button]:hover:!bg-slate-800 [&>button]:!text-slate-500 dark:[&>button]:!text-slate-400 [&>button]:!transition-colors" />
                <MiniMap
                    className="!bg-surface-light dark:!bg-surface-dark !rounded-lg !shadow-lg !border !border-border-light dark:!border-border-dark overflow-hidden"
                    maskColor="rgba(241, 245, 249, 0.6)"
                    nodeColor={() => {
                        return '#e2e8f0';
                    }}
                />

                {/* Top Left Add Button Overlay */}
                <div className="absolute top-6 left-6 z-10 pointer-events-none">
                    <button
                        onClick={handleAddNode}
                        className="pointer-events-auto w-10 h-10 bg-surface-light dark:bg-surface-dark rounded-full shadow-lg border border-border-light dark:border-border-dark flex items-center justify-center text-slate-600 dark:text-slate-300 hover:text-primary transition-colors">
                        <span className="material-icons-round text-2xl">add</span>
                    </button>
                </div>
            </ReactFlow>

            {/* Error Toast */}
            {error && (
                <div className="absolute bottom-6 right-6 z-50 bg-red-100 dark:bg-red-900/30 border border-red-200 dark:border-red-800 text-red-600 dark:text-red-400 px-4 py-3 rounded-lg shadow-lg flex items-center gap-2 animate-in slide-in-from-bottom-2">
                    <span className="material-icons-round">error_outline</span>
                    <p className="text-sm font-medium">{error}</p>
                </div>
            )}
        </main>
    );
};
