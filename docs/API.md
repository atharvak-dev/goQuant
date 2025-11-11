# API Documentation

## Base URL

```
http://localhost:3000
```

## Authentication

All endpoints require authentication via API key or JWT token (implementation specific).

## REST Endpoints

### Upgrade Management

#### Create Upgrade Proposal

```http
POST /upgrade/propose
Content-Type: application/json

{
  "new_program_buffer": "Buffer11111111111111111111111111111111",
  "description": "Upgrade to v2.0.0 with new features"
}
```

**Response:**
```json
{
  "proposal_id": "550e8400-e29b-41d4-a716-446655440000",
  "timelock_until": 1699123456
}
```

#### Approve Upgrade Proposal

```http
POST /upgrade/:id/approve
```

**Response:**
```json
{
  "status": "approved",
  "proposal_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### Execute Upgrade

```http
POST /upgrade/:id/execute
```

**Response:**
```json
{
  "status": "executed",
  "proposal_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### Cancel Upgrade Proposal

```http
POST /upgrade/:id/cancel
```

**Response:**
```json
{
  "status": "cancelled",
  "proposal_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

#### List All Proposals

```http
GET /upgrade/proposals
```

**Response:**
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "proposer": "Proposer11111111111111111111111111111",
    "program": "Program11111111111111111111111111111",
    "new_buffer": "Buffer11111111111111111111111111111111",
    "description": "Upgrade to v2.0.0",
    "proposed_at": 1699000000,
    "timelock_until": 1699123456,
    "approvals": ["Member1...", "Member2..."],
    "approval_threshold": 3,
    "status": "timelock_active",
    "executed_at": null
  }
]
```

#### Get Proposal Status

```http
GET /upgrade/:id/status
```

**Response:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "timelock_active",
  "approvals": 3,
  "threshold": 3,
  "timelock_until": 1699123456,
  "executed_at": null
}
```

### Migration Management

#### Start Migration

```http
POST /migration/start
```

**Response:**
```json
{
  "migration_id": "660e8400-e29b-41d4-a716-446655440001",
  "status": "started"
}
```

#### Get Migration Progress

```http
GET /migration/progress
```

**Response:**
```json
{
  "migration_id": "660e8400-e29b-41d4-a716-446655440001",
  "status": "in_progress",
  "progress_percent": 45.5,
  "migrated_accounts": 455,
  "total_accounts": 1000,
  "failed_accounts": 2,
  "started_at": 1699000000,
  "completed_at": null
}
```

## WebSocket API

### Connection

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');
```

### Message Format

```json
{
  "type": "proposal_created",
  "proposal_id": "550e8400-e29b-41d4-a716-446655440000",
  "message": "New upgrade proposal created",
  "data": {
    "proposer": "Proposer11111111111111111111111111111",
    "new_buffer": "Buffer11111111111111111111111111111111"
  },
  "timestamp": 1699000000
}
```

### Notification Types

- `proposal_created`: New proposal created
- `proposal_approved`: Proposal received approval
- `timelock_expired`: Timelock period expired
- `upgrade_executed`: Upgrade executed successfully
- `migration_progress`: Migration progress update
- `rollback_initiated`: Rollback procedure started

## Error Responses

All errors follow this format:

```json
{
  "error": "Error message description"
}
```

### Error Codes

- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `500 Internal Server Error`: Server error

## Rate Limiting

- 100 requests per minute per IP
- WebSocket connections: 10 per IP

## Examples

### Complete Upgrade Flow

```javascript
// 1. Create proposal
const proposal = await fetch('/upgrade/propose', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    new_program_buffer: 'Buffer111...',
    description: 'Upgrade to v2.0.0'
  })
});

const { proposal_id } = await proposal.json();

// 2. Approve (as multisig member)
await fetch(`/upgrade/${proposal_id}/approve`, {
  method: 'POST'
});

// 3. Wait for timelock (48 hours)
// Monitor via WebSocket or polling

// 4. Execute
await fetch(`/upgrade/${proposal_id}/execute`, {
  method: 'POST'
});
```

### Monitor Migration Progress

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onmessage = (event) => {
  const notification = JSON.parse(event.data);
  
  if (notification.type === 'migration_progress') {
    console.log(`Migration: ${notification.data.progress_percent}%`);
  }
};
```

