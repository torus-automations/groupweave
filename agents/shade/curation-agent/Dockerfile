# Python 3.10 Slim - CPU Optimized
FROM python:3.10-slim

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Pre-download models for TEE isolation (Phi-3 + MiniLM)
# This creates a large image but ensures no runtime internet dependency for models
RUN python -c "from transformers import AutoModelForCausalLM, AutoTokenizer; \
    from sentence_transformers import SentenceTransformer; \
    AutoModelForCausalLM.from_pretrained('microsoft/Phi-3-mini-4k-instruct', trust_remote_code=True); \
    AutoTokenizer.from_pretrained('microsoft/Phi-3-mini-4k-instruct', trust_remote_code=True); \
    SentenceTransformer('all-MiniLM-L6-v2')"

COPY src ./src
COPY data ./data

EXPOSE 3000
ENV PORT=3000

CMD ["python", "src/main.py"]