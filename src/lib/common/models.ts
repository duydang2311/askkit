export enum ChatMessageStatus {
    Pending = 'pending',
    Completed = 'completed',
    Failed = 'failed'
}

export interface Chat {
    created_at: number;
    id: string;
    title: string;
}

export interface ChatMessage {
    created_at: number;
    id: string;
    chatId: string;
    role: string;
    content: string;
    status: ChatMessageStatus;
}

export enum AgentProvider {
    Google = 'google',
    OpenAI = 'open_ai',
}

export interface Agent {
    createdAt: number;
    updatedAt: number;
    id: string;
    provider: AgentProvider;
    model: string;
}
