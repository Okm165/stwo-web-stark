import init, { run_verify } from "stwo-web-stark";

export interface WorkerMessage {
    input: string;
}

export interface WorkerResponse {
    value?: boolean;
    error?: Error;
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
        const response: WorkerResponse = { error: error as Error };
        self.postMessage(response);
    }
};