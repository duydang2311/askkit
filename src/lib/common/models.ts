export enum AgentProvider {
    Gemini = 'gemini',
    OpenAI = 'open_ai',
}

export interface Agent {
    createdAt: number;
    updatedAt: number;
    id: string;
    provider: AgentProvider;
    model: string;
}
