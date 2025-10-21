
```bash
curl -X POST http://localhost:8080/api/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "image": "node:24-alpine3.21",
    "command": "npx",
    "args": ["-y", "@modelcontextprotocol/server-memory"],
    "env_vars": [{"name": "KEY", "value": "VALUE"}]
  }'
```