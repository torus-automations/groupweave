# Use official Node 20 Alpine image
FROM node:20-alpine

# Enable Yarn 4+ corepack
RUN corepack enable

# Set working directory
WORKDIR /app

# Copy package files first for optimal caching
COPY package.json yarn.lock .yarnrc.yml ./

# Install production dependencies (Yarn 4+)
COPY .yarn ./.yarn
RUN yarn install

# Copy application files
COPY . .

# Set non-root user for security
RUN chown -R node:node /app
USER node

# Expose application port
EXPOSE 3000

ENV NODE_ENV=development

# Start command
CMD ["yarn", "start"]