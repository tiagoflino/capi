import { writable } from 'svelte/store';

export interface InferenceMetrics {
    tokens_per_second: number;
    time_to_first_token_ms: number;
    num_output_tokens: number;
    total_context_tokens: number;
}

export const selectedModel = writable<string>('');
export const currentSessionId = writable<string | null>(null);
export const isSidebarOpen = writable<boolean>(true);
export const isGenerating = writable<boolean>(false);
export const inferenceMetrics = writable<InferenceMetrics>({
    tokens_per_second: 0,
    time_to_first_token_ms: 0,
    num_output_tokens: 0,
    total_context_tokens: 0,
});

export const triggerNewChat = writable<number>(0); // Increment to trigger
