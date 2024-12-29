import init, { run_trace_gen } from "stwo-web-stark";

export interface WorkerMessage {
    input: Uint8Array;
}

export interface WorkerResponse {
    execution_resources?: object,
    value?: Uint8Array;
    error?: string;
}

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
    const { input } = event.data;

    try {
        await init();
        const value = await run_trace_gen(input);

        // Send results back to the main thread
        const response: WorkerResponse = { execution_resources: value.execution_resources, value: value.cairo_input };
        self.postMessage(response);
    } catch (error) {
        // Send error back to the main thread
        const response: WorkerResponse = { error: (error as Error).message };
        self.postMessage(response);
    }
};