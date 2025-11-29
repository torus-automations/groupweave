#!/bin/bash
set -e

# Configuration
IMAGE_NAME="groupweave/vlm-classifier-agent"
TAG="latest"

echo "========================================"
echo "BUILDING: $IMAGE_NAME:$TAG"
echo "========================================"

# Build the image
docker build -t $IMAGE_NAME:$TAG .

echo ""
echo "✅ Build complete."
echo "To push to Docker Hub:"
echo "  docker push $IMAGE_NAME:$TAG"
echo ""
echo "To run locally (requires .env file):"
echo "  docker run -p 3001:3001 --env-file .env $IMAGE_NAME:$TAG"
