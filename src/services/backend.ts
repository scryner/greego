import { invoke } from '@tauri-apps/api/core';

export interface NodePosition {
    x: number;
    y: number;
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

    loadGraph: async (canvasId: string = "canvas:main"): Promise<[ChatNode[], FlowEdge[]]> => {
        const [nodes, derives, sequences] = await invoke<[ChatNode[], any[], any[]]>('load_canvas_command', { canvasId });

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

        return [nodes, edges];
    },

    invokeChat: async (canvasId: string, prompt: string): Promise<ChatNode[]> => {
        return await invoke('invoke_chat_command', { canvasId, prompt });
    },

    deleteNode: async (nodeId: string): Promise<void> => {
        return await invoke('delete_node_command', { nodeId });
    },

    moveNodePosition: async (nodeId: string, x: number, y: number): Promise<void> => {
        return await invoke('move_node_position_command', { nodeId, x, y });
    },
};
