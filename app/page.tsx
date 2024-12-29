"use client";

import { useRef, useState } from "react";
import { WorkerMessage as WorkerMessageTraceGen, WorkerResponse as WorkerResponseTraceGen } from "@/worker_trace_gen";
import { WorkerMessage as WorkerMessageProve, WorkerResponse as WorkerResponseProve } from "@/worker_prove";
import { WorkerMessage as WorkerMessageVerify, WorkerResponse as WorkerResponseVerify } from "@/worker_verify";
import { Box, Button, Typography } from "@mui/material";
import CircularProgress from "@mui/material/CircularProgress";
import { useDropzone } from "react-dropzone";

export default function Home() {
  const workerRef = useRef<Worker>(null);
  const [trace, setTrace] = useState<Uint8Array | null>(null);
  const [executionResources, setExecutionResources] = useState<object | null>(null);
  const [proof, setProof] = useState<Uint8Array | null>(null);
  const [verify, setVerify] = useState<boolean | null>(null);

  const [timeTraceGen, setTimeTraceGen] = useState<number | null>(null);
  const [isLoadingTraceGen, setIsLoadingTraceGen] = useState<boolean>(false);

  const [timeProve, setTimeProve] = useState<number | null>(null);
  const [isLoadingProve, setIsLoadingProve] = useState<boolean>(false);

  const [timeVerify, setTimeVerify] = useState<number | null>(null);
  const [isLoadingVerify, setIsLoadingVerify] = useState<boolean>(false);

  const [program, setProgram] = useState<Uint8Array | null>(null);
  const [isLoadingProgram, setIsLoadingProgram] = useState<boolean>(false);

  const [fileName, setFileName] = useState<string | null>(null);
  const [fileSize, setFileSize] = useState<number | null>(null);

  const ondrop = <T extends File>(
    acceptedFiles: T[],
  ) => {
    const file = acceptedFiles[0];
    const reader = new FileReader();

    reader.onload = async (e) => {
      if (e.target && e.target.result) {
        setProgram(new Uint8Array(e.target.result as ArrayBuffer));
        setFileName(file.name);
        setFileSize(file.size);
      }
    };

    reader.readAsArrayBuffer(file);
  };

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop: ondrop,
  });


  function humanFileSize(bytes: number, si = false, dp = 1) {
    const thresh = si ? 1000 : 1024;

    if (Math.abs(bytes) < thresh) {
      return bytes + " B";
    }

    const units = si
      ? ["kB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"]
      : ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
    let u = -1;
    const r = 10 ** dp;

    do {
      bytes /= thresh;
      ++u;
    } while (
      Math.round(Math.abs(bytes) * r) / r >= thresh &&
      u < units.length - 1
    );

    return bytes.toFixed(dp) + " " + units[u];
  }


  const stwo_trace_gen = async () => {
    if (program != null) {
      setIsLoadingTraceGen(true);

      workerRef.current = new Worker(new URL("../worker_trace_gen.ts", import.meta.url), {
        type: "module",
      });

      const startTime = Date.now();

      workerRef.current.onmessage = (event: MessageEvent<WorkerResponseTraceGen>) => {
        const { execution_resources, value, error } = event.data;

        if (error) {
          console.error(error);
        } else if (value && execution_resources) {
          setTrace(value);
          setExecutionResources(execution_resources);
        }

        const endTime = Date.now();
        const elapsedTime = endTime - startTime;
        setTimeTraceGen(elapsedTime);

        workerRef.current?.terminate();

        setIsLoadingTraceGen(false);
      };

      const message: WorkerMessageTraceGen = {
        input: program,
      };

      workerRef.current.postMessage(message);
    }
  };

  const stwo_prove = async () => {
    if (trace != null) {
      setIsLoadingProve(true);

      workerRef.current = new Worker(new URL("../worker_prove.ts", import.meta.url), {
        type: "module",
      });

      const startTime = Date.now();

      workerRef.current.onmessage = (event: MessageEvent<WorkerResponseProve>) => {
        const { value, error } = event.data;

        if (error) {
          console.error(error);
        } else if (value) {
          setProof(value);
        }

        const endTime = Date.now();
        const elapsedTime = endTime - startTime;
        setTimeProve(elapsedTime);

        workerRef.current?.terminate();

        setIsLoadingProve(false);
      };

      const message: WorkerMessageProve = {
        input: trace,
      };

      workerRef.current.postMessage(message);
    }
  };

  const stwo_verify = async () => {
    if (proof != null) {
      setIsLoadingVerify(true);

      workerRef.current = new Worker(new URL("../worker_verify.ts", import.meta.url), {
        type: "module",
      });

      const startTime = Date.now();

      workerRef.current.onmessage = (event: MessageEvent<WorkerResponseVerify>) => {
        const { value, error } = event.data;

        if (error) {
          console.error(error);
        } else if (value) {
          setVerify(value);
        }

        const endTime = Date.now();
        const elapsedTime = endTime - startTime;
        setTimeVerify(elapsedTime);

        workerRef.current?.terminate();

        setIsLoadingVerify(false);
      };

      const message: WorkerMessageVerify = {
        input: proof,
      };

      workerRef.current.postMessage(message);
    }
  };

  return (
    <div className="grid gap-6 p-4 max-w-[800px] m-auto">
      <h1 className="text-2xl font-bold text-center text-gray-300">
        Cairo circuit
      </h1>
      <h1 className="text-2xl font-bold text-center text-gray-300">
        Run - Prove - Verify
      </h1>
      <h1 className="text-2xl font-bold text-center text-gray-300">
        STWO STARK demo
      </h1>

      <br />

      <div
        className="cursor-pointer p-10 border-2 rounded-2xl border-dashed border-gray-800 hover:bg"
        {...getRootProps()}
      >
        <input className="w-full" {...getInputProps()} />
        {fileName != null && fileSize != null ? (
          <p className="text-center">
            {fileName} - {humanFileSize(fileSize)}
          </p>
        ) : isDragActive ? (
          <p className="text-center">Drop the Trace here ...</p>
        ) : (
          <p className="text-center">
            Drag Proof json here, or click to select files
          </p>
        )}
      </div>

      <Button
        sx={{
          color: "#F2A900",
          borderColor: "#473200",
          height: 50,
          "&:hover": {
            borderColor: "#634500",
          },
        }}
        variant="outlined"
        size="small"
        disabled={isLoadingProgram}
        onClick={async () => {
          setIsLoadingProgram(true);
          const response = await fetch("fibonacci_1000.json");

          if (!response.ok) {
            throw new Error(`Failed to fetch file: ${response.status} ${response.statusText}`);
          }

          // Get the file as an ArrayBuffer and convert it to Uint8Array
          const arrayBuffer = await response.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          setProgram(uint8Array);
          setFileName("fibonacci_1000.json");
          setFileSize(uint8Array.length);
          setIsLoadingProgram(false);
        }}
      >
        {isLoadingProgram ? (
          <CircularProgress
            size={24}
            sx={{ color: "#F2A900", animationDuration: "700ms" }}
          />
        ) : (
          <Box display="flex" flexDirection="column" alignItems="center">
            <Typography variant="body2">load fibonacci_1000.json</Typography>
          </Box>
        )}
      </Button>

      <div className="grid grid-flow-row gap-4">
        <Button
          sx={{
            height: 50,
          }}
          variant="outlined"
          size="small"
          disabled={isLoadingTraceGen}
          onClick={async () => {
            stwo_trace_gen();
          }}
        >
          {isLoadingTraceGen ? (
            <CircularProgress
              size={24}
              sx={{ animationDuration: "700ms" }}
            />
          ) : (
            <Box display="flex" flexDirection="column" alignItems="center">
              <Typography variant="body2">trace_gen</Typography>
            </Box>
          )}
        </Button>
        <div className="grid justify-center gap-1 text-xs min-h-6">
          {timeTraceGen !== null ? `Time: ${timeTraceGen / 1000} seconds` : null}
        </div>
        <div className="grid justify-center gap-1 text-xs min-h-6">
          {executionResources !== null ? JSON.stringify(executionResources) : ""}
        </div>
      </div>

      <div className="grid grid-flow-row gap-4">
        <Button
          sx={{
            height: 50,
          }}
          variant="outlined"
          size="small"
          disabled={isLoadingProve}
          onClick={async () => {
            stwo_prove();
          }}
        >
          {isLoadingProve ? (
            <CircularProgress
              size={24}
              sx={{ animationDuration: "700ms" }}
            />
          ) : (
            <Box display="flex" flexDirection="column" alignItems="center">
              <Typography variant="body2">prove</Typography>
            </Box>
          )}
        </Button>
        <div className="grid justify-center gap-1 text-xs min-h-6">
          {timeProve !== null ? `Time: ${timeProve / 1000} seconds` : null}
        </div>
      </div>

      <div className="grid grid-flow-row gap-4">
        <Button
          sx={{
            height: 50,
          }}
          variant="outlined"
          size="small"
          color={verify == null ? "primary" : verify == true ? "success" : "error"}
          disabled={isLoadingVerify}
          onClick={async () => {
            stwo_verify();
          }}
        >
          {isLoadingVerify ? (
            <CircularProgress
              size={24}
              sx={{ animationDuration: "700ms" }}
            />
          ) : (
            <Box display="flex" flexDirection="column" alignItems="center">
              <Typography variant="body2">{verify == null ? "verify" : verify == true ? "proof correct" : "proof wrong"}</Typography>
            </Box>
          )}
        </Button>
        <div className="grid justify-center gap-1 text-xs min-h-6">
          {timeVerify !== null ? `Time: ${timeVerify / 1000} seconds` : null}
        </div>
      </div>

      <textarea
        className="bg-gray-900 text-sm resize-both h-32"
        value={trace?.toString()}
        readOnly
      />
      <textarea
        className="bg-gray-900 text-sm resize-both h-32"
        value={proof?.toString()}
        readOnly
      />
    </div>
  );
}
