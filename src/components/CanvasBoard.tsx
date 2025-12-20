
import {
    ReactFlow,
    Background,
    Controls,
    MiniMap,
    useNodesState,
    useEdgesState,
    useReactFlow,
    type Edge,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';

import { ChatNode, type ChatNodeType } from './ChatNode';
import { PromptInputNode, type PromptInputNodeType } from './PromptInputNode';
import { ModelSelector } from './ModelSelector';
import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { GraphAPI } from '../services/backend';

const nodeTypes = {
    chatNode: ChatNode,
    promptInputNode: PromptInputNode,
};

// App node type (union of all node types)
type AppNodeType = ChatNodeType | PromptInputNodeType;

// Initial Nodes Data
const initialNodes: AppNodeType[] = [];

const initialEdges: Edge[] = [];

// Module-level cache to prevent double-fetching in Strict Mode
// while ensuring the valid component instance receives the data.
let loadGraphPromise: ReturnType<typeof GraphAPI.loadGraph> | null = null;

export const CanvasBoard = () => {
    const [nodes, setNodes, onNodesChange] = useNodesState<AppNodeType>(initialNodes);
    const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);
    const [error, setError] = useState<string | null>(null);

    // Keep track of latest nodes for async callbacks
    const nodesRef = useRef(nodes);
    useEffect(() => {
        nodesRef.current = nodes;
    }, [nodes]);

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

    // Helper to safely convert backend ID to string
    const getSafeId = (id: any): string => {
        if (typeof id === 'string') return id;
        if (typeof id === 'object' && id !== null) {
            // Check for SurrealDB Thing structure
            if ('tb' in id && 'id' in id) {
                let innerId = id.id;
                if (typeof innerId === 'object' && innerId !== null && 'String' in innerId) {
                    innerId = innerId.String;
                } else if (typeof innerId === 'object') {
                    innerId = JSON.stringify(innerId);
                }
                return `${id.tb}:${innerId}`;
            }
            // Fallback for other objects
            return JSON.stringify(id);
        }
        return String(id);
    };

    const handleChatSubmit = async (text: string, tempNodeId: string) => {
        try {
            // Get current position of the temp node before async operation
            const tempNode = nodesRef.current.find(n => n.id === tempNodeId);
            const tempPos = tempNode ? { ...tempNode.position } : null;

            // Invoke backend
            // TODO: Use actual canvas ID. For now hardcoded or passed from somewhere? 
            // The prompt didn't specify multiple canvases, so "main" or similar is fine.
            const canvasId = "canvas:main";
            const newNodes = await GraphAPI.invokeChat(
                canvasId,
                text,
                tempPos ? tempPos.x : 0,
                tempPos ? tempPos.y : 0
            );

            // Transform backend nodes to ReactFlow nodes
            const rfNodes = newNodes.map(transformBackendNode);


            // If we found the temp node, adjust positions of new nodes to match
            if (rfNodes.length > 0 && tempPos) {
                const firstNode = rfNodes[0];
                const dx = tempPos.x - firstNode.position.x;
                const dy = tempPos.y - firstNode.position.y;

                rfNodes.forEach(n => {
                    n.position.x += dx;
                    n.position.y += dy;
                });
            }

            // Replace temp node with real nodes
            setNodes((nds) => {
                const filtered = nds.filter(n => n.id !== tempNodeId);
                return [...filtered, ...rfNodes as any];
            });

            // Handle edges if returned (e.g. if multiple nodes are returned in the future)
            if (newNodes.length >= 2) {
                for (let i = 0; i < newNodes.length - 1; i++) {
                    const source = getSafeId(newNodes[i].id);
                    const target = getSafeId(newNodes[i + 1].id);
                    const newEdge: Edge = {
                        id: `e-${source}-${target}`,
                        source: source,
                        target: target,
                    };
                    setEdges((eds) => [...eds, newEdge]);
                }
            }

        } catch (e) {
            console.error("Failed to invoke chat", e);
            // Error listener will likely catch the emitted error too.
        }
    };

    const transformBackendNode = (n: any): ChatNodeType => {
        const nodeId = getSafeId(n.id);
        const nodeData = (n.type as any)?.data?.value || n.data;

        return {
            id: nodeId,
            position: n.position,
            data: {
                content: nodeData?.text || JSON.stringify(nodeData),
                title: nodeData?.prompt || (nodeData?.role === 'user' ? 'Me' : 'AI'),
                onDelete: handleDeleteNode
            },
            type: 'chatNode'
        };
    };

    const { getViewport, getIntersectingNodes } = useReactFlow();
    const containerRef = useRef<HTMLDivElement>(null);

    const findSmartPosition = (): { x: number, y: number } => {
        if (!containerRef.current) return { x: 100, y: 100 };

        const { width, height } = containerRef.current.getBoundingClientRect();
        const { x: vx, y: vy, zoom } = getViewport();

        // Convert screen dimensions to flow dimensions
        // Visible area in flow coords
        const visibleX = -vx / zoom;
        const visibleY = -vy / zoom;
        const visibleW = width / zoom;
        const visibleH = height / zoom;

        const NODE_WIDTH = 400; // Approx max width
        const NODE_HEIGHT = 200; // Approx height

        // Target: Top-Left area of visible screen (User defined "1st Quadrant")
        // X range: [Left, Center]
        // Y range: [Top, Center]

        // Let's define a grid of potential positions in the visible area
        // Priority: Top-Left -> Center -> Others

        const candidates: { x: number, y: number }[] = [];

        // Top-Left candidates
        // Let's try to place it near the top-left center
        candidates.push({ x: visibleX + visibleW * 0.25 - NODE_WIDTH / 2, y: visibleY + visibleH * 0.25 - NODE_HEIGHT / 2 });
        candidates.push({ x: visibleX + visibleW * 0.4 - NODE_WIDTH / 2, y: visibleY + visibleH * 0.2 - NODE_HEIGHT / 2 });
        candidates.push({ x: visibleX + visibleW * 0.2 - NODE_WIDTH / 2, y: visibleY + visibleH * 0.3 - NODE_HEIGHT / 2 });

        // Center candidates
        candidates.push({ x: visibleX + visibleW / 2 - NODE_WIDTH / 2, y: visibleY + visibleH / 2 - NODE_HEIGHT / 2 });

        // Check for collisions
        for (const pos of candidates) {
            const rect = { ...pos, width: NODE_WIDTH, height: NODE_HEIGHT };
            const collisions = getIntersectingNodes(rect);
            if (collisions.length === 0) {
                return pos;
            }
        }

        // If all fail, try to find *any* open space in visible area with a simple scan
        // Scan 4 quadrants
        const STEPS = 4;
        for (let i = 0; i < STEPS; i++) {
            for (let j = 0; j < STEPS; j++) {
                const x = visibleX + (visibleW / STEPS) * i;
                const y = visibleY + (visibleH / STEPS) * j;
                const rect = { x, y, width: NODE_WIDTH, height: NODE_HEIGHT };
                if (getIntersectingNodes(rect).length === 0) {
                    return { x, y };
                }
            }
        }

        // Fallback: Just offset from center slightly
        return {
            x: visibleX + visibleW / 2 - NODE_WIDTH / 2 + Math.random() * 50,
            y: visibleY + visibleH / 2 - NODE_HEIGHT / 2 + Math.random() * 50
        };
    };

    const handleAddNode = () => {
        const id = `temp-${Date.now()}`;

        // Calculate smart position
        const position = findSmartPosition();

        const newNode: PromptInputNodeType = {
            id,
            position,
            type: 'promptInputNode',
            data: {
                title: 'New chat',
                footer: (
                    <ModelSelector
                        currentModel="lms/gpt-oss-120b"
                        onModelSelect={(model) => console.log("Selected model:", model)}
                    />
                ),
                onDelete: handleDeleteNode,
                onSubmit: handleChatSubmit,
            },
        };
        setNodes((nds) => [...nds, newNode]);
    };

    useEffect(() => {
        let ignore = false;

        const loadGraph = async () => {
            try {
                // TODO: Dynamic canvas ID
                // Use cached promise if available to prevent double-fetching in Strict Mode
                if (!loadGraphPromise) {
                    loadGraphPromise = GraphAPI.loadGraph("canvas:main")
                        .finally(() => {
                            // Clear promise after network completion (but keep result propagation)
                            // We clear it to ensure future full-refreshes (e.g. navigation) get fresh data
                            // The timeout ensures all concurrent effects pick up this promise before it's cleared
                            setTimeout(() => {
                                loadGraphPromise = null;
                            }, 500);
                        });
                }

                const [backendNodes, backendEdges] = await loadGraphPromise;

                if (ignore) return;

                if (backendNodes.length === 0) {
                    // Empty canvas -> Auto create chat node
                    // Verify we haven't already added one (double safety)
                    setNodes(current => {
                        if (current.length > 0) return current;
                        // Need to invoke handleAddNode logic, but since we are inside setNodes, 
                        // we can't call handleAddNode directly as it calls setNodes.
                        // We duplicate the simple creation logic here or refactor.
                        // Ideally, we move initialization out.
                        // For now, let's just use the handleAddNode ONLY if not ignored.
                        return current; // Return current to break the flow, handle via outside
                    });
                    // Use specific call outside setNodes
                    handleAddNode();
                } else {
                    // Transform and set nodes
                    const rfNodes = backendNodes.map(transformBackendNode);
                    setNodes(rfNodes);

                    // Transform and set edges
                    const rfEdges = backendEdges.map(e => ({
                        id: getSafeId(e.id) || `e-${e.source}-${e.target}`,
                        source: getSafeId(e.source),
                        target: getSafeId(e.target)
                    }));
                    setEdges(rfEdges);
                }
            } catch (e) {
                if (!ignore) {
                    console.error("Failed to load graph", e);
                    setError("Failed to load graph data");
                }
            }
        };

        loadGraph();

        return () => {
            ignore = true;
        };
    }, []);

    return (
        <main ref={containerRef} className="flex-1 h-screen w-full bg-background-light dark:bg-background-dark relative">
            <ReactFlow
                nodes={nodes}
                edges={edges}
                onNodesChange={onNodesChange}
                onEdgesChange={onEdgesChange}
                nodeTypes={nodeTypes}
                onNodeDragStop={(_, node) => {
                    const id = getSafeId(node.id);
                    // Don't save temp nodes
                    if (!id.startsWith('temp-')) {
                        GraphAPI.moveNodePosition(id, node.position.x, node.position.y)
                            .catch(e => console.error("Failed to save node position", e));
                    }
                }}

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
