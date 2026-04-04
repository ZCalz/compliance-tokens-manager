#!/bin/bash
# Blocks writes to sensitive files before Claude edits them.
# Exit 2 = block the action and show message to user.
# Exit 0 = allow the action.

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE_PATH" ]; then
  exit 0
fi

PROTECTED_PATTERNS=(
  ".env"
  ".env.local"
  ".env.production"
  "*.pem"
  "*.key"
  "secrets/"
  ".git/"
)

for pattern in "${PROTECTED_PATTERNS[@]}"; do
  case "$FILE_PATH" in
    *"$pattern"*)
      echo "Blocked: '$FILE_PATH' matches protected pattern '$pattern'" >&2
      exit 2
      ;;
  esac
done

exit 0
