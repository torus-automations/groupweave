# Use official Node 20 Alpine image
FROM node:20-alpine



# Set working directory
WORKDIR /app

# Copy package files first for optimal caching
COPY package.json pnpm-lock.yaml ./

# Install pnpm and production dependencies
RUN npm install -g pnpm && pnpm install --frozen-lockfile --prod

# Copy application files
COPY . .

# Set non-root user for security
RUN chown -R node:node /app
USER node

# Expose application port
EXPOSE 3000

ENV NODE_ENV=development

# Start command
CMD ["pnpm", "start"]