import init, { run_verify } from "stwo-web-stark";

export interface WorkerMessage {
    input: Uint8Array;
}

export interface WorkerResponse {
    value?: boolean;
    error?: string;
}

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
    const { input } = event.data;

    try {
        await init();
        const value = await run_verify(input);

        // Send results back to the main thread
        const response: WorkerResponse = { value: value };
        self.postMessage(response);
    } catch (error) {
        // Send error back to the main thread
        const response: WorkerResponse = { error: (error as Error).message };
        self.postMessage(response);
    }
};