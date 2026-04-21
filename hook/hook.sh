#!/bin/bash
# Breather. Claude Code hook.
# Sends a lightweight HTTP POST to the Breather app on each conversation turn (Stop event).

INPUT=$(cat)
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" -m 1 -X POST http://127.0.0.1:17422/event \
  -H "Content-Type: application/json" \
  -d "$INPUT" 2>/dev/null)

if [ "$RESPONSE" != "200" ]; then
  echo '{"additionalContext": "[Breather] App is not running. Your coding session is not being tracked. Open the Breather app to stay protected."}'
fi

exit 0
