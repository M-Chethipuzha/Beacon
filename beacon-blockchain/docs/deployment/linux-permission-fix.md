# BEACON Blockchain - Linux Deployment Guide

## Permission Error Fix

If you encounter this error on Linux:

```
Error: Storage error: Failed to create database directory: Permission denied (os error 13)
```

This is due to Docker volume permission mismatches. Here are the solutions:

## Quick Fix (Recommended)

### 1. Run the setup script:

```bash
# Make script executable
chmod +x scripts/setup-linux.sh

# Run setup
./scripts/setup-linux.sh
```

### 2. Start the services:

```bash
docker-compose up -d
```

## Manual Fix

### 1. Create and set permissions for data directories:

```bash
# Create directories
mkdir -p ./data/node-{1,2,3}
mkdir -p ./logs/node-{1,2,3}

# Set proper ownership (UID 1000 matches Docker user)
sudo chown -R 1000:1000 ./data/
sudo chown -R 1000:1000 ./logs/
chmod -R 755 ./data/
chmod -R 755 ./logs/
```

### 2. Alternative: Run with privileged mode

If you still have issues, you can temporarily run with privileged mode by adding to docker-compose.yml:

```yaml
services:
  beacon-node-1:
    privileged: true
    # ... rest of configuration
```

### 3. Alternative: Use init containers

Add an init container to fix permissions:

```yaml
services:
  beacon-node-1-init:
    image: alpine:latest
    user: root
    volumes:
      - beacon-node-1-data:/data/beacon/storage
      - beacon-node-1-logs:/data/beacon/logs
    command: >
      sh -c "
        mkdir -p /data/beacon/storage /data/beacon/logs &&
        chown -R 1000:1000 /data/beacon &&
        chmod -R 755 /data/beacon
      "

  beacon-node-1:
    depends_on:
      - beacon-node-1-init
    # ... rest of configuration
```

## Verification

### Check if containers are running:

```bash
docker-compose ps
```

### Check logs for permission errors:

```bash
docker-compose logs beacon-node-1
```

### Test API endpoint:

```bash
curl http://localhost:3001/health
```

## Troubleshooting

### If you still get permission errors:

1. **Check file ownership:**

   ```bash
   ls -la ./data/node-1/
   ```

2. **Check container user:**

   ```bash
   docker-compose exec beacon-node-1 id
   ```

3. **Run with temporary root access:**

   ```bash
   docker-compose exec --user root beacon-node-1 bash
   # Inside container:
   chown -R beacon:beacon /data/beacon
   exit
   ```

4. **Restart services:**
   ```bash
   docker-compose down
   docker-compose up -d
   ```

### SELinux Issues (CentOS/RHEL/Fedora):

If using SELinux, you may need to set the correct context:

```bash
sudo setsebool -P container_manage_cgroup true
sudo semanage fcontext -a -t container_file_t "./data(/.*)?"
sudo restorecon -R ./data
```

## Production Recommendations

1. **Use specific UIDs:** Always use consistent UID/GID (1000:1000) across environments
2. **Directory structure:** Keep data and logs in separate volumes
3. **Backup strategy:** Ensure data directories are properly backed up
4. **Security:** Run containers as non-root user (already implemented)
5. **Monitoring:** Use health checks and log monitoring

## Files Modified

The following files have been updated to fix permission issues:

1. `Dockerfile` - Added specific UID/GID and entrypoint script
2. `docker-compose.yml` - Added user specification and explicit volume bindings
3. `docker/entrypoint.sh` - Permission fixing script
4. `scripts/setup-linux.sh` - Automated setup script

## Testing

After applying fixes, test with:

```bash
# Start services
docker-compose up -d

# Wait for services to start
sleep 30

# Test each node
for i in {1..3}; do
  echo "Testing node $i..."
  curl -s http://localhost:300$i/health | jq '.'
done

# Check storage directories
for i in {1..3}; do
  echo "Node $i storage:"
  ls -la ./data/node-$i/
done
```
