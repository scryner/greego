import { invoke } from '@tauri-apps/api/core';

export interface NodePosition {
    x: number;
    y: number;
}

export interface Canvas {
    id?: string | object;
    title: string;
    created_at: string;
    embedding_id?: string;
    reranker_id?: string;
    chat_model_id?: string;
}

// Matching Rust struct ChatNode
// pub struct ChatNode {
//     pub id: Option<Thing>,
//     pub position: NodePosition,
//     pub data: serde_json::Value,
//     pub type: String,
// }
// Thing is usually serialized as a string "table:id" or object. 
// For frontend simplicity we often treat it as string or unknown.
// Let's use string for ID if it comes back as string, or keep it flexible.
export interface ChatNode {
    id?: string | object;
    position: NodePosition;
    data: any;
    type: string;
}

// Matching Rust struct FlowEdge
// pub struct FlowEdge {
//     pub id: Option<Thing>,
//     pub source: String,
//     pub target: String,
// }
export interface FlowEdge {
    id?: string | object;
    source: string;
    target: string;
}

export const GraphAPI = {
    saveNode: async (node: ChatNode): Promise<ChatNode> => {
        return await invoke('save_node_command', { node });
    },

    connectEdge: async (edgeId: string | null, source: string, target: string): Promise<FlowEdge> => {
        return await invoke('connect_edge_command', { edgeId, source, target });
    },

    loadGraph: async (canvasId: string = "canvas:main"): Promise<{ nodes: ChatNode[], edges: FlowEdge[], canvas: Canvas | null }> => {
        const data = await invoke<{
            canvas: Canvas,
            nodes: ChatNode[],
            derives: any[],
            sequences: any[]
        } | null>('load_canvas_command', { canvasId });

        if (!data) {
            console.warn(`Canvas ${canvasId} not found`);
            return { nodes: [], edges: [], canvas: null };
        }

        const { canvas, nodes, derives, sequences } = data;

        const edges: FlowEdge[] = [
            ...derives.map(d => ({
                id: d.id,
                source: typeof d.from === 'object' && d.from.id ? `${d.from.tb}:${d.from.id}` : d.from,
                target: typeof d.to === 'object' && d.to.id ? `${d.to.tb}:${d.to.id}` : d.to,
            })),
            ...sequences.map(s => ({
                id: s.id,
                source: typeof s.from === 'object' && s.from.id ? `${s.from.tb}:${s.from.id}` : s.from,
                target: typeof s.to === 'object' && s.to.id ? `${s.to.tb}:${s.to.id}` : s.to,
            }))
        ];

        return { nodes, edges, canvas };
    },

    invokeChat: async (canvasId: string, prompt: string, model: string, x: number, y: number, parentId?: string, relationType?: 'sequence' | 'derive'): Promise<ChatNode[]> => {
        return await invoke('invoke_chat_command', { canvasId, prompt, modelId: model, x, y, parentId, relationType });
    },

    deleteNode: async (nodeId: string): Promise<void> => {
        return await invoke('delete_node_command', { nodeId });
    },

    moveNodePosition: async (nodeId: string, x: number, y: number): Promise<void> => {
        return await invoke('move_node_position_command', { nodeId, x, y });
    },

    updateCanvasChatModel: async (canvasId: string, modelId: string | null): Promise<Canvas> => {
        return await invoke('update_canvas_chat_model_command', { canvasId, modelId });
    },

    unifiedQuery: async (prompt: string, modelId: string, canvasId?: string): Promise<void> => {
        console.log("Invoking unified_query_command with:", { prompt, modelId, canvasId });
        return await invoke('unified_query_command', { prompt, modelId, canvasId });
    },
};

// Helper to safely convert backend ID to string
export const getSafeId = (id: any): string => {
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
