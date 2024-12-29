# Use an official Rust base image with Node.js installed
FROM node:slim

# Set the working directory
WORKDIR /usr/src/app

# Copy package.json and package-lock.json
COPY package*.json ./

# Install dependencies with fallback
RUN npm cache clean --force && npm config set registry http://registry.npmjs.org/ && npm install

# Copy the rest of the application code
COPY . .

# Build Rust code with wasm-pack
RUN cd backend/out && npm link

# Link the Rust-built package
RUN npm link binius-web-snark

# Build the Next.js application
RUN npm run build

# Expose the port the app runs on
EXPOSE 3000

# Start the application
CMD ["npm", "start"]
