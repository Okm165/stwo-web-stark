import init, { run_trace_gen } from "stwo-web-stark";

export interface WorkerMessage {
    input: string;
}

export interface WorkerResponse {
    execution_resources?: string,
    prover_input?: string;
    error?: Error;
}

self.onmessage = async (event: MessageEvent<WorkerMessage>) => {
    const { input } = event.data;

    try {
        await init();
        const value = await run_trace_gen(input);

        // Send results back to the main thread
        const response: WorkerResponse = { execution_resources: value.execution_resources, prover_input: value.prover_input };
        self.postMessage(response)
    } catch (error) {
        // Send error back to the main thread
        const response: WorkerResponse = { error: error as Error };
        self.postMessage(response);
    }
};