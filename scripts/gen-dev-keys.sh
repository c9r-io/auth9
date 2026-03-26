#!/bin/bash
# Generate development JWT keys for local development and QA tools.
#
# Generates a fresh RSA 2048 key pair and outputs:
#   1. deploy/dev-certs/jwt/private.key  (for QA tools)
#   2. Escaped PEM strings suitable for .env (printed to stdout)
#
# If .env exists and JWT_PRIVATE_KEY is empty, the keys are written into .env automatically.
#
# Idempotent: skips generation if the key file already exists.
# Use --force to regenerate.

set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

FORCE=false
for arg in "$@"; do
  case $arg in
    --force) FORCE=true ;;
  esac
done

JWT_DIR="$PROJECT_DIR/deploy/dev-certs/jwt"
PRIVATE_KEY="$JWT_DIR/private.key"

if [ -f "$PRIVATE_KEY" ] && [ "$FORCE" = false ]; then
  echo "JWT dev key already exists: $PRIVATE_KEY (use --force to regenerate)"
  exit 0
fi

mkdir -p "$JWT_DIR"

# Generate a fresh RSA 2048 key pair
echo "Generating RSA 2048 key pair..."
openssl genrsa 2048 2>/dev/null | openssl pkcs8 -topk8 -nocrypt -out "$PRIVATE_KEY" 2>/dev/null
chmod 600 "$PRIVATE_KEY"

# Verify the key is valid
if ! openssl rsa -in "$PRIVATE_KEY" -check -noout 2>/dev/null; then
  echo "ERROR: Generated key failed validation"
  rm -f "$PRIVATE_KEY"
  exit 1
fi

# Extract public key
PUBLIC_KEY_FILE="$JWT_DIR/public.key"
openssl rsa -in "$PRIVATE_KEY" -pubout -out "$PUBLIC_KEY_FILE" 2>/dev/null

echo "JWT dev key generated: $PRIVATE_KEY"
echo "JWT public key generated: $PUBLIC_KEY_FILE"

# Auto-populate .env if JWT_PRIVATE_KEY is empty
ENV_FILE="$PROJECT_DIR/.env"
if [ -f "$ENV_FILE" ]; then
  CURRENT_PRIVATE=$(grep '^JWT_PRIVATE_KEY=' "$ENV_FILE" | cut -d= -f2- || true)
  if [ -z "$CURRENT_PRIVATE" ] || [ "$FORCE" = true ]; then
    # Use python3 to read PEM files and write escaped \n values into .env
    # (shell variables and sed can't reliably handle literal \n in PEM values)
    python3 -c '
import sys
private_key_path, public_key_path, env_path = sys.argv[1], sys.argv[2], sys.argv[3]

with open(private_key_path) as f:
    escaped_private = f.read().strip().replace("\n", "\\n")
with open(public_key_path) as f:
    escaped_public = f.read().strip().replace("\n", "\\n")

with open(env_path, "r") as f:
    lines = f.readlines()

result = []
replaced = {"JWT_PRIVATE_KEY": False, "JWT_PUBLIC_KEY": False}
for line in lines:
    if line.startswith("JWT_PRIVATE_KEY="):
        result.append(f"JWT_PRIVATE_KEY={escaped_private}\n")
        replaced["JWT_PRIVATE_KEY"] = True
    elif line.startswith("JWT_PUBLIC_KEY="):
        result.append(f"JWT_PUBLIC_KEY={escaped_public}\n")
        replaced["JWT_PUBLIC_KEY"] = True
    else:
        result.append(line)

if not replaced["JWT_PRIVATE_KEY"]:
    result.append(f"JWT_PRIVATE_KEY={escaped_private}\n")
if not replaced["JWT_PUBLIC_KEY"]:
    result.append(f"JWT_PUBLIC_KEY={escaped_public}\n")

with open(env_path, "w") as f:
    f.writelines(result)
' "$PRIVATE_KEY" "$PUBLIC_KEY_FILE" "$ENV_FILE"
    echo "Updated .env with new JWT keys"
  else
    echo "Skipping .env update (JWT_PRIVATE_KEY already set; use --force to overwrite)"
  fi
fi
