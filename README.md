# Project Overview

This project provides web demo of proving Cairo programs with [STWO](https://github.com/starkware-libs/stwo) prover (WIP).

## Getting Started

### Build the WASM Backend

For detailed instructions, refer to the [backend README](./backend/README.md).
Backend [NPM package](https://www.npmjs.com/package/stwo-web-stark)

### Run the Development Server

1. Start the server using Docker Compose:
   ```bash
   docker compose up
   ```

2. Open your browser and navigate to [http://localhost:3000](http://localhost:3000) to view the application.

## Acknowledgments

This project incorporates the following libraries and tools:

- [cairo-vm](https://github.com/lambdaclass/cairo-vm) by [Lambdaclass](https://github.com/lambdaclass)
- [stwo-cairo](https://github.com/starkware-libs/stwo-cairo) by [StarkWare](https://github.com/starkware-libs)
- [stwo](https://github.com/starkware-libs/stwo) by [StarkWare](https://github.com/starkware-libs)

