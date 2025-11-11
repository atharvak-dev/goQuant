# Operational Runbook

## Overview

This runbook provides step-by-step procedures for operating the GoQuant Upgrade Management System.

## Prerequisites

- Access to multisig wallet
- Solana CLI tools installed
- Database access
- Backend service running
- Anchor development environment

## Common Operations

### Proposing an Upgrade

1. **Build New Program**
   ```bash
   cd programs/upgrade-manager
   anchor build
   ```

2. **Deploy Program Buffer**
   ```bash
   solana program deploy target/deploy/upgrade_manager.so \
     --program-id <PROGRAM_ID> \
     --buffer <BUFFER_KEYPAIR>
   ```

3. **Create Proposal via API**
   ```bash
   curl -X POST http://localhost:3000/upgrade/propose \
     -H "Content-Type: application/json" \
     -d '{
       "new_program_buffer": "<BUFFER_PUBKEY>",
       "description": "Upgrade description"
     }'
   ```

4. **Verify Proposal Created**
   - Check database: `SELECT * FROM upgrade_proposals WHERE proposal_id = '<ID>';`
   - Check on-chain: Query proposal account

### Approving a Proposal

1. **Review Proposal**
   - Check proposal details via API: `GET /upgrade/:id/status`
   - Review code changes
   - Verify program buffer

2. **Approve as Multisig Member**
   ```bash
   curl -X POST http://localhost:3000/upgrade/:id/approve
   ```

3. **Monitor Approval Progress**
   - Watch for threshold to be met
   - Timelock will activate automatically

### Executing an Upgrade

1. **Verify Requirements**
   - Timelock has expired
   - Sufficient approvals (3/5)
   - Program buffer verified

2. **Execute Upgrade**
   ```bash
   curl -X POST http://localhost:3000/upgrade/:id/execute
   ```

3. **Verify Execution**
   - Check transaction signature
   - Verify program upgraded on-chain
   - Check program version

4. **Post-Upgrade Verification**
   - Run health checks
   - Verify critical functions
   - Monitor error rates

### Starting a Migration

1. **Verify Upgrade Completed**
   - Ensure upgrade executed successfully
   - Identify accounts needing migration

2. **Start Migration**
   ```bash
   curl -X POST http://localhost:3000/migration/start
   ```

3. **Monitor Progress**
   - Watch migration progress: `GET /migration/progress`
   - Monitor via WebSocket for real-time updates
   - Check database for detailed status

4. **Verify Completion**
   - Ensure all accounts migrated
   - Verify data integrity
   - Check for failed migrations

### Emergency Rollback

1. **Detect Upgrade Failure**
   - Monitor error rates
   - Check program health
   - Verify user funds

2. **Initiate Rollback**
   ```rust
   // Via backend service
   rollback_handler.rollback_program(old_program_id).await?;
   ```

3. **Rollback Steps**
   - Pause system operations
   - Close all positions
   - Return user funds
   - Deploy old program
   - Resume operations

4. **Post-Rollback**
   - Verify system operational
   - Analyze failure cause
   - Document incident

## Monitoring

### Key Metrics

- Proposal status counts
- Approval rates
- Timelock expiration times
- Migration progress
- Error rates
- Upgrade success/failure rates

### Health Checks

```bash
# Check backend service
curl http://localhost:3000/health

# Check database connection
psql -d goquant_upgrades -c "SELECT COUNT(*) FROM upgrade_proposals;"

# Check Solana RPC
solana cluster-version
```

### Logs

- Backend logs: `backend/logs/`
- Database logs: PostgreSQL logs
- Solana logs: `solana.log`

## Troubleshooting

### Proposal Not Creating

- Check multisig member permissions
- Verify program buffer exists
- Check database connection
- Review backend logs

### Approval Not Registering

- Verify multisig member
- Check proposal status
- Ensure not already approved
- Review transaction signature

### Upgrade Execution Failing

- Verify timelock expired
- Check approval threshold met
- Verify program buffer
- Check upgrade authority
- Review Solana transaction logs

### Migration Stuck

- Check migration progress
- Review failed accounts
- Verify account access
- Check migration logic
- Review error logs

## Maintenance

### Database Maintenance

```sql
-- Clean old proposals (older than 90 days)
DELETE FROM upgrade_proposals 
WHERE created_at < NOW() - INTERVAL '90 days' 
AND status = 'executed';

-- Archive old migrations
INSERT INTO migration_archive 
SELECT * FROM migration_progress 
WHERE completed_at < NOW() - INTERVAL '30 days';
```

### Backup Procedures

1. **Database Backup**
   ```bash
   pg_dump goquant_upgrades > backup_$(date +%Y%m%d).sql
   ```

2. **Program Backup**
   - Store program binaries
   - Keep buffer accounts
   - Document program versions

### Updates

1. **Backend Service Update**
   - Deploy new version
   - Run database migrations
   - Restart service
   - Verify functionality

2. **Program Update**
   - Follow upgrade procedure
   - Test on devnet first
   - Deploy to mainnet

## Security Procedures

### Access Control

- Multisig members only can propose/approve
- API authentication required
- Database access restricted
- Audit all operations

### Incident Response

1. **Detect Incident**
   - Monitor alerts
   - Review logs
   - Verify reports

2. **Contain**
   - Pause operations if needed
   - Isolate affected systems
   - Preserve evidence

3. **Remediate**
   - Fix issue
   - Verify fix
   - Resume operations

4. **Post-Incident**
   - Document incident
   - Analyze root cause
   - Update procedures

## Emergency Contacts

- On-call engineer: [Contact]
- Multisig members: [List]
- Security team: [Contact]

