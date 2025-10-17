#!/bin/bash

# Ollama Qwen3-Coder 30B Integration Script
# Usage: ./scripts/ollama/qwen3-coder.sh "Your prompt here"

MODEL="qwen3-coder:30b"
OLLAMA_URL="http://localhost:11434"

# Check if Ollama is running
if ! curl -s "$OLLAMA_URL/api/version" > /dev/null; then
    echo "❌ Ollama is not running. Please start Ollama first:"
    echo "   ollama serve"
    exit 1
fi

# Check if model is available
if ! ollama list | grep -q "$MODEL"; then
    echo "📥 Pulling $MODEL model..."
    ollama pull "$MODEL"
fi

PROMPT="$1"

if [ -z "$PROMPT" ]; then
    echo "❌ Usage: $0 \"Your prompt here\""
    echo ""
    echo "Examples:"
    echo "  $0 \"Generate a Rust function for validating TOML configuration\""
    echo "  $0 \"Review this code for best practices: [paste code]\""
    echo "  $0 \"Explain how this algorithm works\""
    exit 1
fi

echo "🤖 Querying $MODEL..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create the JSON payload
JSON_PAYLOAD=$(cat << EOF
{
  "model": "$MODEL",
  "prompt": "$PROMPT",
  "stream": false,
  "options": {
    "temperature": 0.7,
    "top_p": 0.9,
    "num_predict": 2048
  }
}
EOF
)

# Make the API call and extract response
RESPONSE=$(curl -s -X POST "$OLLAMA_URL/api/generate" \
  -H "Content-Type: application/json" \
  -d "$JSON_PAYLOAD")

# Extract response using a reliable method
RESPONSE_TEXT=$(echo "$RESPONSE" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    print(data.get('response', ''))
except:
    # Fallback to grep if Python fails
    import subprocess
    result = subprocess.run(['grep', '-o', '\"response\":\"[^\"]*\"'], 
                          input=sys.stdin.read(), text=True, capture_output=True)
    if result.returncode == 0:
        line = result.stdout.strip().split('\"')[-2]
        print(line)
" 2>/dev/null || echo "$RESPONSE" | grep -o '"response":"[^"]*"' | head -1 | sed 's/"response":"//' | sed 's/"$//')

# Format and display response
if [ -n "$RESPONSE_TEXT" ] && [ "$RESPONSE_TEXT" != "null" ]; then
  echo "$RESPONSE_TEXT" | sed 's/^/│ /' | sed 's/$/ │/'
else
  echo "│ No response received from model │"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Response from $MODEL complete"
