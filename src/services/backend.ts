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

    loadGraph: async (): Promise<[ChatNode[], FlowEdge[]]> => {
        return await invoke('load_board_command');
    },
};
