# CENTRA-NF v1.0.0 - Deployment Operations Manual
## Production Deployment & Operations Playbook

**Version**: 1.0.0  
**Last Updated**: 2026-03-16  
**Audience**: DevOps, SRE, Operations Teams  

---

## 📋 Table of Contents

1. Pre-Deployment Checklist
2. Staging Deployment (Phase 1)
3. Production Deployment (Phase 2)
4. Health Verification
5. Monitoring & Alerts
6. Incident Response
7. Rollback Procedures
8. Scaling & Performance Tuning
9. Disaster Recovery
10. Maintenance Windows

---

## 1. PRE-DEPLOYMENT CHECKLIST

### Infrastructure Requirements

```bash
# Minimum infrastructure needed
RESOURCE_REQUIREMENTS=(
  "Kubernetes 1.27+ (or Docker 24.0+)"
  "x86_64 CPU with AVX2 support"
  "8GB RAM per pod instance"
  "20GB disk per pod"
  "1Gbps network connectivity"
  "TLS certificates (valid for 1+ year)"
  "Log aggregation system (ELK/Loki)"
  "Monitoring system (Prometheus)"
)

# Network prerequisites
NETWORK_REQS=(
  "HTTPS/TLS enabled"
  "Ingress controller configured"
  "Service mesh (optional: Istio)"
  "Network policy defined"
  "Load balancer ready"
)

# Database/Storage prerequisites
STORAGE_REQS=(
  "PersistentVolume provisioner ready"
  "PostgreSQL database (optional)"
  "Redis cache layer (optional)"
  "Backups configured"
)
```

### Pre-Flight Checks

```bash
#!/bin/bash
DATE=$(date "+%Y-%m-%d %H:%M:%S")
echo "=== Pre-Flight Checks [$DATE] ===" 

# 1. Verify Kubernetes cluster
kubectl cluster-info
kubectl get nodes
echo "✓ Kubernetes cluster reachable"

# 2. Check available resources
kubectl describe nodes | grep -E "Allocated resources|Requests"
echo "✓ Sufficient cluster resources"

# 3. Verify image registry access
docker pull $(docker inspect centra-nf:1.0.0 2>/dev/null) || docker login -u USER -p PASS REGISTRY
echo "✓ Registry access verified"

# 4. Test secret access
kubectl get secret centra-nf-secrets
echo "✓ Secrets available"

# 5. Verify monitoring is ready
# Check Prometheus, Grafana connectivity
echo "✓ Monitoring systems ready"

# 6. Backup current state
kubectl get all --all-namespaces > deployment_backup_$(date +%Y%m%d_%H%M%S).txt
echo "✓ Pre-deployment backup created"
```

---

## 2. STAGING DEPLOYMENT (Phase 1)

### 2.1 Create Staging Namespace

```bash
# Create isolated staging environment
kubectl create namespace centra-nf-staging

# Verify namespace
kubectl get namespace centra-nf-staging
```

### 2.2 Deploy Configuration

```yaml
# Save as k8s/deployment-staging.yaml
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: centra-nf-config
  namespace: centra-nf-staging
data:
  LOG_LEVEL: "INFO"
  RUST_LOG: "centra_nf=debug"
  ENVIRONMENT: "staging"
  WORKERS: "2"

---
apiVersion: v1
kind: Secret
metadata:
  name: centra-nf-secrets
  namespace: centra-nf-staging
type: Opaque
stringData:
  AES_KEY: "$(openssl rand -hex 32)"  # Generate fresh key
  SIGNING_KEY: "$(openssl rand -hex 32)"

---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: centra-nf-staging
  namespace: centra-nf-staging
  labels:
    app: centra-nf
    environment: staging
    version: v1.0.0
spec:
  replicas: 2  # Start with 2 for HA testing
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: centra-nf
      environment: staging
  template:
    metadata:
      labels:
        app: centra-nf
        environment: staging
        version: v1.0.0
    spec:
      containers:
      - name: centra-nf
        image: registry.example.com/centra-nf:1.0.0
        imagePullPolicy: IfNotPresent
        ports:
        - name: http
          containerPort: 8080
        - name: metrics
          containerPort: 9090
        
        # Environment variables
        env:
        - name: ENVIRONMENT
          valueFrom:
            configMapKeyRef:
              name: centra-nf-config
              key: ENVIRONMENT
        - name: LOG_LEVEL
          valueFrom:
            configMapKeyRef:
              name: centra-nf-config
              key: LOG_LEVEL
        - name: AES_KEY
          valueFrom:
            secretKeyRef:
              name: centra-nf-secrets
              key: AES_KEY
        
        # Resource limits
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "512Mi"
            cpu: "1000m"
        
        # Health checks
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        
        # Graceful shutdown
        lifecycle:
          preStop:
            exec:
              command: ["/bin/sh", "-c", "sleep 15"]
      
      # Pod disruption budget
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - centra-nf
              topologyKey: kubernetes.io/hostname

---
apiVersion: v1
kind: Service
metadata:
  name: centra-nf-staging
  namespace: centra-nf-staging
  labels:
    app: centra-nf
spec:
  type: ClusterIP
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: metrics
    port: 9090
    targetPort: 9090
  selector:
    app: centra-nf
    environment: staging

---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: centra-nf-staging-hpa
  namespace: centra-nf-staging
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: centra-nf-staging
  minReplicas: 2
  maxReplicas: 5
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### 2.3 Deploy to Staging

```bash
# Deploy
kubectl apply -f k8s/deployment-staging.yaml

# Monitor deployment rollout
kubectl rollout status deployment/centra-nf-staging -n centra-nf-staging

# Verify pods running
kubectl get pods -n centra-nf-staging -o wide

# Check logs
kubectl logs -f deployment/centra-nf-staging -n centra-nf-staging

# Port forward for testing
kubectl port-forward svc/centra-nf-staging -n centra-nf-staging 8080:80
```

### 2.4 Staging Validation

```bash
#!/bin/bash
echo "=== Staging Validation ==="

STAGING_POD=$(kubectl get pod -n centra-nf-staging -o jsonpath='{.items[0].metadata.name}')
POD_IP=$(kubectl get pod $STAGING_POD -n centra-nf-staging -o jsonpath='{.status.podIP}')

# 1. Health Check
echo "1. Health Check:"
curl -s http://$POD_IP:8080/health | jq .
echo ""

# 2. Version Check
echo "2. Version Check:"
curl -s http://$POD_IP:8080/version | jq .
echo ""

# 3. Metrics Check
echo "3. Metrics Available:"
curl -s http://$POD_IP:9090/metrics | head -20
echo ""

# 4. Performance Baseline
echo "4. Load Test (10 concurrent):"
ab -c 10 -n 100 http://$POD_IP:8080/health
echo ""

# 5. Memory usage
echo "5. Resource Usage:"
kubectl top pods -n centra-nf-staging
echo ""

# 6. Verify no errors in logs
echo "6. Error Check (should be empty):"
kubectl logs deployment/centra-nf-staging -n centra-nf-staging | grep -i error | head -5
echo "[END]"
```

---

## 3. PRODUCTION DEPLOYMENT (Phase 2)

### 3.1 Production Namespace & Secrets

```bash
# Create production namespace
kubectl create namespace centra-nf-prod

# Set resource quotas
kubectl apply -f - <<EOF
apiVersion: v1
kind: ResourceQuota
metadata:
  name: centra-nf-prod-quota
  namespace: centra-nf-prod
spec:
  hard:
    requests.cpu: "10"
    requests.memory: 20Gi
    limits.cpu: "20"
    limits.memory: 40Gi
    pods: "50"
EOF

# Verify quota
kubectl describe quota -n centra-nf-prod
```

### 3.2 Production Deployment

```bash
# Create production config (similar to staging, but with replicas: 5)
# Save as k8s/deployment-prod.yaml
# (Modify staging deployment with: replicas: 5, maxReplicas: 20)

# Apply production deployment
kubectl apply -f k8s/deployment-prod.yaml

# Monitor rollout
kubectl rollout status deployment/centra-nf -n centra-nf-prod

# Verify pods
kubectl get pods -n centra-nf-prod -o wide
kubectl top pods -n centra-nf-prod
```

### 3.3 Ingress Configuration

```yaml
# k8s/ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: centra-nf-ingress
  namespace: centra-nf-prod
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - centra-nf.example.com
    secretName: centra-nf-tls
  rules:
  - host: centra-nf.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: centra-nf
            port:
              number: 80
```

---

## 4. HEALTH VERIFICATION

### 4.1 Endpoint Checks

```bash
#!/bin/bash
PROD_URL="https://centra-nf.example.com"

echo "=== Health Verification ==="

# 1. Primary health endpoint
echo "1. Service Health:"
curl -s -o /dev/null -w "%{http_code}\n" $PROD_URL/health
# Expected: 200

# 2. Readiness check
echo "2. Readiness:"
curl -s -o /dev/null -w "%{http_code}\n" $PROD_URL/ready
# Expected: 200

# 3. Liveness check
echo "3. Liveness:"
curl -s -o /dev/null -w "%{http_code}\n" $PROD_URL/alive
# Expected: 200

# 4. Metrics availability
echo "4. Metrics Scrape:"
curl -s -o /dev/null -w "%{http_code}\n" $PROD_URL/metrics
# Expected: 200
```

### 4.2 Pod Readiness

```bash
# Wait for all pods ready
kubectl wait --for=condition=Ready pod -l app=centra-nf -n centra-nf-prod --timeout=300s

# Verify all pods are running
kubectl get pods -n centra-nf-prod --no-headers | awk '{print $3}' | sort | uniq -c
# Expected: All pods "Running"

# Check container logs for errors
kubectl logs -l app=centra-nf -n centra-nf-prod --tail=50 | grep -i error
# Expected: No errors
```

---

## 5. MONITORING & ALERTS

### 5.1 Prometheus Configuration

```yaml
# prometheus-config.yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s
  
scrape_configs:
- job_name: 'centra-nf'
  static_configs:
  - targets: ['centra-nf.example.com/metrics']
  
alert_rules:
- name: centra-nf-alerts
  rules:
  - alert: HighErrorRate
    expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.01
    for: 5m
    labels:
      severity: critical
    
  - alert: PodCrashLooping
    expr: rate(kube_pod_container_status_restarts_total[5m]) > 0
```

### 5.2 Grafana Dashboards

```bash
# Import prebuilt dashboard
curl -X POST http://grafana:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @grafana-dashboard.json
```

### 5.3 Log Aggregation

```bash
# Verify logs are flowing to ELK/Loki
curl -s http://elasticsearch:9200/centra-nf-logs-*/_count | jq .count

# Example query: Find errors in last hour
curl -s "http://elasticsearch:9200/centra-nf-logs-*/_search?q=level:ERROR&size=100"
```

---

## 6. INCIDENT RESPONSE

### 6.1 Service Down - Checklist

```bash
#!/bin/bash
echo "=== INCIDENT: Service Down ==="

# 1. Check pod status
kubectl get pods -n centra-nf-prod -o wide

# 2. Check recent events
kubectl describe deployment centra-nf -n centra-nf-prod | grep Events -A 20

# 3. Check logs for errors
kubectl logs --all-containers=true -l app=centra-nf -n centra-nf-prod --tail=200

# 4. Check resource constraints
kubectl top nodes
kubectl top pods -n centra-nf-prod

# 5. Check disk space
df -h /var/lib/kubelet/pods

# 6. Restart pods (if needed)
kubectl rollout restart deployment/centra-nf -n centra-nf-prod

# 7. Monitor recovery
kubectl rollout status deployment/centra-nf -n centra-nf-prod --timeout=5m

# 8. Verify service restored
curl -s https://centra-nf.example.com/health | jq .
```

### 6.2 High Latency - Diagnostics

```bash
# 1. Monitor query latency
watch 'curl -s https://centra-nf.example.com/metrics | grep latency'

# 2. Check pod CPU/memory
kubectl top pods -n centra-nf-prod

# 3. Check network metrics
kubectl exec -it $POD_NAME -n centra-nf-prod -- ss -s

# 4. Scale up if needed
kubectl scale deployment/centra-nf -n centra-nf-prod --replicas=8
```

---

## 7. ROLLBACK PROCEDURES

### 7.1 Quick Rollback (Previous Version)

```bash
# Find previous revision
kubectl rollout history deployment/centra-nf -n centra-nf-prod

# Rollback to previous
kubectl rollout undo deployment/centra-nf -n centra-nf-prod

# Monitor rollback
kubectl rollout status deployment/centra-nf -n centra-nf-prod

# Verify service recovered
curl -s https://centra-nf.example.com/health
```

### 7.2 Blue-Green Deployment (Alternative)

```bash
# Current: centra-nf-v1.0.0 (production)
# New: centra-nf-v1.0.1 (staged)

# 1. Deploy new version alongside
kubectl create deployment centra-nf-v1.0.1 \
  --image=registry.example.com/centra-nf:1.0.1 \
  -n centra-nf-prod

# 2. Switch traffic (update service selector)
kubectl patch service centra-nf -n centra-nf-prod \
  -p '{"spec":{"selector":{"version":"v1.0.1"}}}'

# 3. If issues, switch back
kubectl patch service centra-nf -n centra-nf-prod \
  -p '{"spec":{"selector":{"version":"v1.0.0"}}}'

# 4. Delete old version
kubectl delete deployment centra-nf-v1.0.0 -n centra-nf-prod
```

---

## 8. SCALING & PERFORMANCE TUNING

### 8.1 Horizontal Scaling

```bash
# Manual scale
kubectl scale deployment/centra-nf --replicas=10 -n centra-nf-prod

# Auto-scale range
kubectl patch hpa centra-nf-hpa -n centra-nf-prod -p \
  '{"spec":{"maxReplicas":20, "minReplicas":3}}'

# Monitor scaling events
kubectl logs deployment/metrics-server -n kube-system | grep scaling
```

### 8.2 Resource Optimization

```bash
# Increase CPU for faster compilation
kubectl patch deployment/centra-nf -n centra-nf-prod --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/template/spec/containers/0/resources/requests/cpu",
    "value":"1000m"
  }
]'

# Increase memory for large programs
kubectl patch deployment/centra-nf -n centra-nf-prod --type='json' -p='[
  {
    "op": "replace",
    "path": "/spec/template/spec/containers/0/resources/limits/memory",
    "value":"2Gi"
  }
]'
```

---

## 9. DISASTER RECOVERY

### 9.1 Data Backup

```bash
# Backup configuration
kubectl get all -n centra-nf-prod -o yaml > centra-nf-backup-$(date +%Y%m%d).yaml

# Backup secrets
kubectl get secrets -n centra-nf-prod -o yaml > centra-nf-secrets-$(date +%Y%m%d).yaml

# Schedule daily backup
(crontab -l 2>/dev/null; echo "0 2 * * * kubectl get all -n centra-nf-prod -o yaml > /backups/centra-nf-\$(date +\%Y\%m\%d).yaml") | crontab -
```

### 9.2 Full Cluster Failure Recovery

```bash
# 1. Restore from backup
kubectl apply -f centra-nf-backup-20260316.yaml

# 2. Restore secrets
kubectl apply -f centra-nf-secrets-20260316.yaml

# 3. Wait for pods
kubectl wait --for=condition=Ready pod -l app=centra-nf -n centra-nf-prod --timeout=300s

# 4. Verify health
curl -s https://centra-nf.example.com/health
```

---

## 10. MAINTENANCE WINDOWS

### 10.1 Scheduled Maintenance

```bash
# 1. Announce maintenance window
echo "Maintenance window: 2026-03-20 02:00-03:00 UTC"

# 2. Enable maintenance mode (optional service)
kubectl patch svc centra-nf -n centra-nf-prod -p \
  '{"metadata":{"annotations":{"maintenance":"true"}}}'

# 3. Drain nodes gracefully
kubectl drain NODE_NAME --ignore-daemonsets --delete-emptydir-data

# 4. Perform maintenance (OS updates, etc.)
# ... system administration tasks ...

# 5. Uncordon node
kubectl uncordon NODE_NAME

# 6. Verify service recovered
curl -s https://centra-nf.example.com/health
```

### 10.2 Upgrade Path

```bash
# 1. Test new version in staging
kubectl apply -f k8s/deployment-staging-v1.0.1.yaml

# 2. Validate in staging (run all tests)
# ...

# 3. Roll out new version with canary
kubectl set image deployment/centra-nf \
  centra-nf=registry.example.com/centra-nf:1.0.1 \
  -n centra-nf-prod --record

# 4. Monitor metrics
watch 'kubectl top pods -n centra-nf-prod'

# 5. If issues detected, rollback
kubectl rollout undo deployment/centra-nf -n centra-nf-prod
```

---

## ✅ Operator Checklist

Before Go-Live:

```
□ Pre-deployment infrastructure verified
□ Staging deployment successful
□ All staging tests passed
□ Production secrets configured
□ Monitoring system ready (Prometheus/Grafana)
□ Log aggregation configured
□ Backup procedures tested
□ Rollback tested
□ Team trained
□ Runbooks reviewed
□ On-call contact established
□ Escalation path clear
```

---

## 📞 Emergency Contacts

```
On-Call SRE:     +1-XXX-XXX-XXXX
Escalation:      team@centra-nf.org
Incident Channel: #centra-nf-incidents
```

---

**Status**: Production Deployment Ready  
**Next Step**: Execute Pre-Deployment Checklist (Section 1)  
**Estimated Go-Live**: 2026-03-20

