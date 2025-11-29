# Python 3.10 Slim - CPU optimized
FROM python:3.10-slim

WORKDIR /app

# Install system dependencies (cv2 requires libgl1)
RUN apt-get update && apt-get install -y \
    libgl1-mesa-glx \
    libglib2.0-0 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements and install
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Pre-download model weights (baking them into the image for TEE isolation)
# This increases build time but ensures the agent is self-contained
RUN python -c "from transformers import Qwen2VLForConditionalGeneration, AutoProcessor; \
    Qwen2VLForConditionalGeneration.from_pretrained('Qwen/Qwen3-VL-2B-Instruct'); \
    AutoProcessor.from_pretrained('Qwen/Qwen3-VL-2B-Instruct')"

COPY data ./data
COPY src ./src

EXPOSE 3001
ENV PORT=3001

# Run the application
CMD ["python", "src/main.py"]