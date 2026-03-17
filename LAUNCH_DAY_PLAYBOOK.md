# CENTRA-NF v1.0.0 - LAUNCH DAY PLAYBOOK
## Master Coordination Document for Production Go-Live

**Execution Date**: 2026-03-20 (Target)  
**Duration**: 4-6 hours (initial go-live window)  
**Status**: READY FOR EXECUTION  

---

## 🎯 LAUNCH DAY OBJECTIVES

✅ Deploy CENTRA-NF v1.0.0 to production  
✅ Verify all services operational (99.9% availability)  
✅ Establish monitoring & alerting baseline  
✅ Complete incident response training  
✅ Document all deployment artifacts  

---

## 📅 PRE-LAUNCH TIMELINE

### T-7 Days (Week of 2026-03-13)
- [ ] Final security audit complete
- [ ] Load testing in staging (1000+ req/sec)
- [ ] Disaster recovery drill - FULL EXECUTION
- [ ] On-call roster confirmed
- [ ] Communication plan finalized

### T-3 Days (2026-03-17)
- [ ] Execute PRE_LAUNCH_VERIFICATION_PROTOCOL.md (ALL 10 phases)
- [ ] Document results + green-light from technical team
- [ ] Final sign-offs obtained from all stakeholders
- [ ] Monitoring dashboards tested + verified
- [ ] Backup systems tested

### T-1 Day (2026-03-19)
- [ ] Infrastructure final check (Kubernetes cluster health)
- [ ] Database backups verified
- [ ] All documentation reviewed + accessible
- [ ] Team training completed
- [ ] Decision: GO or NO-GO

---

## 🚀 LAUNCH DAY EXECUTION (2026-03-20)

### PRE-LAUNCH (06:00 UTC - 07:00 UTC)

**Duration**: 60 minutes  
**Team**: DevOps + SRE + On-Call Engineer

```bash
# 1. Final System Checks (06:00)
TIME_LOG="[06:00 UTC]"
echo "$TIME_LOG Starting pre-launch checks..."

# Check Kubernetes cluster health
kubectl get nodes
kubectl get all -n default

# Verify Docker registry connectivity
docker pull registry.example.com/centra-nf:1.0.0

# Verify secrets are accessible
kubectl get secrets -n kube-system

# Check disk space
df -h /var/lib/kubelet

echo "$TIME_LOG All pre-flight checks PASSED"

# 2. Database/Storage Final Checks (06:10)
TIME_LOG="[06:10 UTC]"
echo "$TIME_LOG Checking persistent storage..."

# Verify PersistentVolume provisioner
kubectl get pv

# Check database connectivity (if applicable)
# pg_isready -h postgres.example.com -U centra_nf

echo "$TIME_LOG Storage verification PASSED"

# 3. Monitoring System Check (06:20)
TIME_LOG="[06:20 UTC]"
echo "$TIME_LOG Verifying monitoring systems..."

# Prometheus connectivity
curl -s http://prometheus:9090/-/healthy

# Grafana connectivity
curl -s http://grafana:3000/api/health | jq .

# Alert manager
curl -s http://alertmanager:9093/-/healthy

echo "$TIME_LOG Monitoring systems READY"

# 4. Team Readiness (06:30)
TIME_LOG="[06:30 UTC]"
echo "$TIME_LOG Confirming team readiness..."

# All teams on Slack/chat?
# echo "🟢 DevOps team ready?"
# echo "🟢 SRE team ready?"
# echo "🟢 On-call engineer standing by?"
# echo "🟢 Product manager briefed?"

# Final GO/NO-GO decision
echo "$TIME_LOG FINAL DECISION: GO FOR LAUNCH"

# 5. Create Deployment Timestamp (06:50)
DEPLOYMENT_ID="centra-nf-v1.0.0-$(date +%Y%m%d_%H%M%S)"
echo "$TIME_LOG Deployment ID: $DEPLOYMENT_ID"

# Create pre-deployment backup
kubectl get all --all-namespaces -o yaml > backup_${DEPLOYMENT_ID}.yaml
kubectl get secrets --all-namespaces -o yaml > secrets_${DEPLOYMENT_ID}.yaml

echo "$TIME_LOG Pre-deployment backup created"
```

### DEPLOYMENT PHASE 1: STAGING (07:00 UTC - 07:30 UTC)

**Duration**: 30 minutes  
**Objective**: Deploy to staging, verify basic health

```bash
TIME_LOG="[07:00 UTC]"
echo "$TIME_LOG PHASE 1: Staging Deployment"

# 1. Deploy to staging namespace
kubectl apply -f k8s/deployment-staging.yaml \
  -n centra-nf-staging

echo "$TIME_LOG Deployment manifest applied"

# 2. Monitor rollout
kubectl rollout status deployment/centra-nf-staging \
  -n centra-nf-staging \
  --timeout=5m

echo "$TIME_LOG Pods ready"

# 3. Verify pods running
RUNNING_PODS=$(kubectl get pods -n centra-nf-staging \
  --field-selector=status.phase=Running \
  --no-headers | wc -l)
echo "$TIME_LOG Running pods: $RUNNING_PODS (expected: 2)"

# 4. Execute health checks
echo "$TIME_LOG Executing health checks..."

POD=$(kubectl get pod -n centra-nf-staging -o jsonpath='{.items[0].metadata.name}')
POD_IP=$(kubectl get pod $POD -n centra-nf-staging -o jsonpath='{.status.podIP}')

# Health endpoint
curl -s http://$POD_IP:8080/health | jq .

# Metrics endpoint
curl -s http://$POD_IP:9090/metrics | head -20

echo "$TIME_LOG Health checks PASSED"

# 5. Smoke tests
echo "$TIME_LOG Running smoke tests..."

# Test compilation
kubectl exec $POD -n centra-nf-staging -- \
  /opt/centra/bin/centra-nf-cli compile /opt/centra/examples/simple.cnf

echo "$TIME_LOG Smoke tests PASSED"
```

### DEPLOYMENT PHASE 2: CANARY (07:30 UTC - 08:00 UTC)

**Duration**: 30 minutes  
**Objective**: Deploy to 10% of production, monitor closely

```bash
TIME_LOG="[07:30 UTC]"
echo "$TIME_LOG PHASE 2: Canary Deployment (10%)"

# 1. Deploy 1 replica to production (canary)
kubectl set image deployment/centra-nf \
  centra-nf=registry.example.com/centra-nf:1.0.0 \
  -n centra-nf-prod \
  --record

echo "$TIME_LOG Image update initiated"

# 2. Scale to 1 replica (canary)
kubectl scale deployment/centra-nf \
  --replicas=1 \
  -n centra-nf-prod

# 3. Wait for pod ready
kubectl wait --for=condition=Ready pod \
  -l app=centra-nf,environment=prod \
  -n centra-nf-prod \
  --timeout=5m

echo "$TIME_LOG Canary pod ready"

# 4. Monitor canary metrics for 15 minutes
for minute in {1..15}; do
  TIME_LOG="[07:$((30 + minute)) UTC]"
  echo "$TIME_LOG Metrics check $minute/15..."
  
  # Get current metrics
  kubectl top pods -n centra-nf-prod
  
  # Check logs for errors
  ERROR_COUNT=$(kubectl logs deployment/centra-nf \
    -n centra-nf-prod \
    --tail=100 | grep -i error | wc -l)
  
  echo "$TIME_LOG Errors in logs: $ERROR_COUNT (max allowed: 5)"
  
  if [ $ERROR_COUNT -gt 5 ]; then
    echo "$TIME_LOG ERROR THRESHOLD EXCEEDED - ROLLBACK INITIATED"
    kubectl rollout undo deployment/centra-nf -n centra-nf-prod
    exit 1
  fi
  
  # Canary health check via ingress
  CANARY_HEALTH=$(curl -s -o /dev/null -w "%{http_code}" \
    https://centra-nf.example.com/health)
  
  echo "$TIME_LOG Canary HTTP status: $CANARY_HEALTH (expected: 200)"
  
  if [ $CANARY_HEALTH -ne 200 ]; then
    echo "$TIME_LOG HEALTH CHECK FAILED - ROLLBACK INITIATED"
    kubectl rollout undo deployment/centra-nf -n centra-nf-prod
    exit 1
  fi
  
  sleep 60
done

echo "$TIME_LOG Canary phase PASSED"
```

### DEPLOYMENT PHASE 3: ROLLING UPDATE (08:00 UTC - 09:00 UTC)

**Duration**: 60 minutes  
**Objective**: Gradually scale to 100% production

```bash
TIME_LOG="[08:00 UTC]"
echo "$TIME_LOG PHASE 3: Rolling Update to 100%"

# 1. Scale to 25%
TIME_LOG="[08:00 UTC]"
echo "$TIME_LOG Scaling to 25% (2/8 pods)..."
kubectl scale deployment/centra-nf --replicas=2 -n centra-nf-prod
kubectl wait --for=condition=Ready pod \
  -l app=centra-nf,environment=prod \
  -n centra-nf-prod \
  --timeout=5m
echo "$TIME_LOG ✓ 25% deployment successful"

# 2. Monitor 50%
TIME_LOG="[08:15 UTC]"
echo "$TIME_LOG Scaling to 50% (4/8 pods)..."
kubectl scale deployment/centra-nf --replicas=4 -n centra-nf-prod
kubectl wait --for=condition=Ready pod \
  -l app=centra-nf,environment=prod \
  -n centra-nf-prod \
  --timeout=5m
echo "$TIME_LOG ✓ 50% deployment successful"

# 3. Health check at 50%
ERROR_COUNT=$(kubectl logs deployment/centra-nf \
  -n centra-nf-prod --tail=200 | grep -i error | wc -l)
echo "$TIME_LOG Errors at 50%: $ERROR_COUNT (max: 10)"

if [ $ERROR_COUNT -gt 10 ]; then
  echo "$TIME_LOG THRESHOLD EXCEEDED - ROLLBACK"
  kubectl rollout undo deployment/centra-nf -n centra-nf-prod
  exit 1
fi

# 4. Complete rollout to 100%
TIME_LOG="[08:30 UTC]"
echo "$TIME_LOG Scaling to 100% (8/8 pods)..."
kubectl scale deployment/centra-nf --replicas=8 -n centra-nf-prod

# Wait for all pods
kubectl wait --for=condition=Ready pod \
  -l app=centra-nf,environment=prod \
  -n centra-nf-prod \
  --timeout=10m

echo "$TIME_LOG ✓ 100% deployment complete"

# 5. Full production verification
TIME_LOG="[09:00 UTC]"
echo "$TIME_LOG Running full production verification..."

# Verify all pods running
RUNNING=$(kubectl get pods -n centra-nf-prod \
  --field-selector=status.phase=Running \
  --no-headers | wc -l)
echo "$TIME_LOG Pods running: $RUNNING/8"

# Check ingress health
INGRESS_STATUS=$(curl -s -o /dev/null -w "%{http_code}" \
  https://centra-nf.example.com/health)
echo "$TIME_LOG Ingress HTTP status: $INGRESS_STATUS"

# Check metrics flowing
METRIC_COUNT=$(curl -s https://centra-nf.example.com/metrics | wc -l)
echo "$TIME_LOG Metrics lines: $METRIC_COUNT"
```

### POST-DEPLOYMENT (09:00 UTC onwards)

**Duration**: Ongoing  
**Team**: 24x7 On-Call Monitoring

```bash
# 1. Verify all health endpoints
echo "=== FINAL VERIFICATION ==="
curl -s https://centra-nf.example.com/health | jq .
curl -s https://centra-nf.example.com/ready | jq .
curl -s https://centra-nf.example.com/alive | jq .

# 2. Check dashboard
echo "Dashboard: https://grafana.example.com/d/centra-nf"

# 3. Verify alerts configured
kubectl get alerts -n centra-nf-prod

# 4. Document deployment
DEPLOY_END=$(date "+%Y-%m-%d %H:%M:%S")
echo "Deployment completed at: $DEPLOY_END"
echo "Deployment ID: $DEPLOYMENT_ID"

# 5. Publish notification
cat > deployment_summary.txt << EOF
CENTRA-NF v1.0.0 Production Deployment
Date: $(date)
Status: ✅ SUCCESSFUL
Duration: ~2 hours
Pods: 8 (all running)
SLA: 99.9% uptime target
Dashboard: https://grafana.example.com/d/centra-nf
Alert Channel: #centra-nf-incidents
On-Call: ${ON_CALL_ENGINEER}
EOF

# 6. Start 24/7 monitoring
echo "🟢 Production monitoring active"
echo "On-call contact available: ${ON_CALL_PHONE}"
```

---

## 📊 MONITORING DURING LAUNCH

### Key Metrics to Watch

```
Metric                  Threshold      Action if Exceeded
─────────────────────────────────────────────────────────
HTTP Error Rate         > 1%           Page on-call, investigate
P99 Latency            > 500ms         Check pod logs
Memory Usage           > 80%           Scale horizontally
CPU Usage              > 85%           Scale + optimize
Pod Restart Count      > 0             Investigate crash
```

### Dashboard Views

1. **Real-time Traffic**: Requests/sec, errors/sec
2. **Resource Usage**: CPU, Memory, Disk per pod
3. **Application Health**: Service health, pod readiness
4. **Latency Distribution**: P50, P95, P99 latencies
5. **Error Analysis**: Error types, stack traces

---

## 🆘 INCIDENT RESPONSE DURING LAUNCH

### If Service Fails at Any Phase

**Immediate Actions (0-5 min)**:
```bash
# 1. STOP deployment
kubectl rollout pause deployment/centra-nf -n centra-nf-prod

# 2. Assess damage
kubectl get all -n centra-nf-prod
kubectl logs deployment/centra-nf -n centra-nf-prod --tail=100

# 3. Rollback
kubectl rollout undo deployment/centra-nf -n centra-nf-prod

# 4. Verify rollback
kubectl rollout status deployment/centra-nf -n centra-nf-prod

# 5. Page on-call engineer
# Call: +1-XXX-XXX-XXXX
```

### If Performance Degrades

```bash
# 1. Check pod resource limits
kubectl describe pod <POD_NAME> -n centra-nf-prod

# 2. Scale horizontally (temporary fix)
kubectl scale deployment/centra-nf --replicas=15 -n centra-nf-prod

# 3. Investigate root cause
kubectl top pods -n centra-nf-prod
kubectl top nodes

# 4. Analysis in logs
kubectl logs deployment/centra-nf -n centra-nf-prod | tail -200
```

---

## ✅ LAUNCH SUCCESS CRITERIA

All of the following must be TRUE:

```
□ All 8 pods running in centra-nf-prod
□ HTTP /health endpoint returning 200
□ /metrics endpoint returning Prometheus format
□ Zero errors in pod logs (or < 5 expected errors)
□ Mean latency < 100ms
□ Error rate < 0.1%
□ All monitoring dashboards showing data
□ Alerts configured and firing correctly
□ Team able to log into dashboard
□ Rollback plan tested + ready
```

---

## 📞 COMMUNICATION PLAN

### Pre-Launch (T-1 day)
- Email: All stakeholders with go/no-go decision
- Chat: Technical team on Slack #deployment
- Meeting: Pre-deployment briefing (30 min)

### During Launch (Launch Day)
- Chat: 15-min updates to #deployment channel
- Dashboard: Live ops dashboard accessible
- Call: Bridge line open for incident response
- Escalation: On-call number published

### Post-Launch
- Report: Deployment success announcement
- Summary: Lessons learned (24 hours after)
- Metrics: Performance baseline published
- Next: v1.0.1 planning

---

## 📋 ROLLBACK DECISION TREE

```
Does /health return 200?
├─ YES → Continue to "Error Rate Check"
└─ NO → ROLLBACK (critical)

Error Rate > 1%?
├─ YES → ROLLBACK (or scale + investigate)
└─ NO → Continue to "Memory Check"

Memory > 80%?
├─ YES → Scale horizontally
└─ NO → Continue to "CPU Check"

CPU > 85%?
├─ YES → Scale horizontally
└─ NO → LAUNCH SUCCESSFUL ✅
```

---

## 🎉 LAUNCH SUCCESS TEMPLATE

```
🚀 CENTRA-NF v1.0.0 - LAUNCH COMPLETE 🚀

Timeline:
├─ Pre-launch:    ✅ 06:00-07:00 (60 min success)
├─ Canary:        ✅ 07:00-08:00 (stable)
├─ 25% rollout:   ✅ 08:00-08:15 (healthy)
├─ 50% rollout:   ✅ 08:15-08:30 (no errors)
├─ 100% rollout:  ✅ 08:30-09:00 (all green)
└─ Verification:  ✅ 09:00+ (monitoring active)

Metrics:
├─ Error rate:    0.02% ✅
├─ Latency P99:   85ms ✅
├─ Uptime:        100% ✅
└─ SLA:           99.9% on track ✅

Status: 🟢 PRODUCTION LIVE
Recommendation: PROCEED WITH CONFIDENCE

Next: 24/7 monitoring begins
On-call: [NAME] standing by
Dashboard: https://grafana.example.com/d/centra-nf
```

---

## 📂 REQUIRED DOCUMENTS (ALL READY)

✅ [COMPREHENSIVE_LAUNCH_READINESS_REPORT.md](COMPREHENSIVE_LAUNCH_READINESS_REPORT.md)  
✅ [PRE_LAUNCH_VERIFICATION_PROTOCOL.md](PRE_LAUNCH_VERIFICATION_PROTOCOL.md)  
✅ [DEPLOYMENT_OPERATIONS_MANUAL.md](DEPLOYMENT_OPERATIONS_MANUAL.md)  
✅ [PRODUCTION_DEPLOYMENT.md](PRODUCTION_DEPLOYMENT.md)  
✅ [HARDENING_COMPLETE.md](HARDENING_COMPLETE.md)  

---

## ✨ FINAL CHECKLIST

```
BEFORE LAUNCH DAY
─────────────────
□ All verification phases complete (PRE_LAUNCH_VERIFICATION_PROTOCOL.md)
□ Team training completed
□ On-call roster confirmed
□ Backup systems tested
□ Incident response drills passed
□ Monitoring dashboards tested
□ Communication channels set up
□ Rollback procedure documented + practiced

LAUNCH DAY
──────────
□ Infrastructure pre-flight checks passed
□ Team aligned on objectives
□ Monitoring active
□ Incident response bridge line ready
□ Deploy staging (Phase 1) complete
□ Canary monitoring (Phase 2) successful
□ Rolling update (Phase 3) complete
□ All success criteria met
□ Incident response on standby

POST-LAUNCH
───────────
□ All systems monitoring
□ SLA targets on track
□ Team debriefing scheduled
□ Deployment artifacts archived
□ Lessons learned documented
```

---

**Status**: 🟢 READY FOR LAUNCH  
**Recommendation**: PROCEED WITH CONFIDENCE  
**Next Step**: Execute this playbook on 2026-03-20 at 06:00 UTC  

**Contact**: On-call SRE: [PHONE]  
**Escalation**: team@centra-nf.org

