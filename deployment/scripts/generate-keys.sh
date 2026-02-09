#!/bin/bash
# Generate validator keys for BachLedger multi-node deployment

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYS_DIR="${SCRIPT_DIR}/../keys"

echo "=========================================="
echo "BachLedger Validator Key Generator"
echo "=========================================="

# Create keys directory
mkdir -p "$KEYS_DIR"

# Number of validators
NUM_VALIDATORS=${1:-4}

echo "Generating keys for $NUM_VALIDATORS validators..."
echo ""

for i in $(seq 1 $NUM_VALIDATORS); do
    KEY_FILE="$KEYS_DIR/validator${i}.key"

    if [ -f "$KEY_FILE" ]; then
        echo "Warning: $KEY_FILE already exists, skipping..."
    else
        # Generate 32-byte random private key
        openssl rand -hex 32 > "$KEY_FILE"
        chmod 600 "$KEY_FILE"
        echo "Generated: validator${i}.key"
    fi
done

echo ""
echo "=========================================="
echo "Keys generated in: $KEYS_DIR"
echo ""
echo "IMPORTANT: Keep these keys secure!"
echo "Never commit them to version control."
echo "=========================================="

# Create .gitignore in keys directory
cat > "$KEYS_DIR/.gitignore" << 'EOF'
# Ignore all key files
*.key
*.pem
*.json
!.gitignore
EOF

echo ""
echo "Created $KEYS_DIR/.gitignore to prevent accidental commits"
