# LLM Gateway — Deployment Architecture

**File:** `docs/deployment.md`  
**Version:** 1.0  
**Status:** Draft  
**Project:** `llm-gateway`  
**Implementation:** Rust + Python  
**Deployment:** Docker-first

---

# 1. Purpose

This document defines the deployment architecture and operational procedures for `llm-gateway`.

The deployment strategy must support:

- Reproducible local development.
- Isolated infrastructure dependencies.
- Secure provider credential management.
- Production-like staging.
- Horizontal gateway scaling.
- Reliable database migrations.
- Monitoring and alerting.
- Backup and recovery.
- Controlled application releases.

The initial deployment uses Docker Compose. Kubernetes is a potential future deployment target, not a requirement for the first release.

# 2. Deployment Objectives

| ID | Objective | Description |
|---|---|---|
| DEP-001 | Reproducibility | Consistent deployment environments |
| DEP-002 | Isolation | Separate application and infrastructure services |
| DEP-003 | Security | No secrets embedded in images or source code |
| DEP-004 | Scalability | Support multiple gateway instances |
| DEP-005 | Reliability | Health checks and graceful shutdown |
| DEP-006 | Observability | Centralized logs, metrics, and traces |
| DEP-007 | Recoverability | Database backup and restoration |
| DEP-008 | Automation | Repeatable CI/CD deployment process |

# 3. Deployment Environments

| Environment | Purpose | Characteristics |
|---|---|---|
| Local | Development | Docker Compose, mock providers |
| CI | Automated validation | Disposable dependencies |
| Staging | Pre-production | Production-like configuration |
| Production | Real workloads | Managed infrastructure and controlled releases |

## 3.1 Environment Separation

Each environment must have separate:

- PostgreSQL databases.
- Redis instances or namespaces with appropriate isolation.
- Provider credentials.
- API keys.
- Encryption keys.
- Observability access policies.
- Budget and rate-limit configurations.

Production credentials must never be reused in development or CI.

# 4. Deployment Architecture

## 4.1 Initial Docker Architecture

The initial deployment uses separate containers for application and infrastructure responsibilities.

```mermaid
flowchart TB
    CLIENT["Client Applications"] --> PROXY["Reverse Proxy / TLS"]
    PROXY --> GW["LLM Gateway - Rust / Axum"]

    subgraph APP["Application Containers"]
        GW
        WORKER["Evaluation Worker - Python"]
        MIGRATE["Migration Job"]
    end

    subgraph INFRA["Infrastructure Containers"]
        PG[("PostgreSQL")]
        REDIS[("Redis")]
        BROKER["Event Broker / Redis Streams"]
        OTEL["OpenTelemetry Collector"]
        PROM["Prometheus"]
        GRAFANA["Grafana"]
        LANGFUSE["Langfuse - Optional"]
    end

    GW --> PG
    GW --> REDIS
    GW --> BROKER
    GW --> OTEL
    WORKER --> PG
    WORKER --> BROKER
    WORKER --> OTEL
    OTEL --> PROM
    PROM --> GRAFANA
    OTEL --> LANGFUSE

    GW --> PROVIDERS["External LLM Providers"]
```

The diagram represents logical responsibilities. Some components may share infrastructure in the initial development environment.

## 4.2 Container Responsibilities

| Container | Responsibility |
|---|---|
| `gateway` | HTTP API, authentication, routing, inference |
| `evaluation-worker` | Asynchronous evaluation |
| `postgres` | Persistent relational data |
| `redis` | Rate limiting, concurrency, exact cache |
| `event-broker` | Asynchronous event delivery |
| `otel-collector` | Telemetry collection and export |
| `prometheus` | Metrics storage |
| `grafana` | Dashboards and alerts |
| `langfuse` | Optional LLM observability interface |
| `migrate` | Database schema migrations |

## 4.3 Initial Infrastructure Simplification

For local development, Redis Streams can serve as the initial event transport if the selected implementation supports the required delivery and recovery semantics.

Do not introduce Kafka solely for architectural appearance.

If workload, retention, consumer isolation, or throughput requirements later justify it, introduce a dedicated broker through an explicit architecture decision.

# 5. Repository Deployment Structure

Suggested repository structure:

```text
llm-gateway/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── pyproject.toml
├── uv.lock
├── Dockerfile
├── Dockerfile.worker
├── docker-compose.yml
├── docker-compose.staging.yml
├── .dockerignore
├── .env.example
│
├── crates/
│   ├── gateway-api/
│   ├── gateway-core/
│   ├── identity/
│   ├── authorization/
│   ├── policy-engine/
│   ├── model-router/
│   ├── provider-core/
│   ├── provider-registry/
│   ├── provider-adapters/
│   ├── budget-manager/
│   ├── rate-limiter/
│   ├── cache-engine/
│   ├── reliability-engine/
│   ├── telemetry/
│   └── shared-kernel/
│
├── services/
│   └── evaluation-worker/
│
├── config/
│   ├── models.yaml
│   ├── routing.yaml
│   └── policies.yaml
│
├── migrations/
│
├── deploy/
│   ├── postgres/
│   ├── redis/
│   ├── prometheus/
│   ├── grafana/
│   ├── otel/
│   └── scripts/
│
├── docs/
└── tests/
```

This is a proposed structure. Reconcile crate names with the actual workspace and avoid duplicating existing modules.

# 6. Docker Image Strategy

## 6.1 Gateway Image

Use a multi-stage Docker build.

Stages:

1. Dependency preparation.
2. Rust compilation.
3. Minimal runtime image.

Requirements:

- Pin the Rust toolchain.
- Use `Cargo.lock`.
- Use a non-root runtime user.
- Exclude source files and build artifacts from the runtime image.
- Do not include provider credentials.
- Use a minimal runtime dependency set.
- Run vulnerability scans.

Illustrative Dockerfile:

```dockerfile
FROM rust:1-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release --workspace

FROM debian:bookworm-slim AS runtime

RUN useradd --system --uid 10001 gateway

WORKDIR /app

COPY --from=builder /app/target/release/gateway-api /usr/local/bin/gateway

USER gateway

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/gateway"]
```

The final build must match the actual workspace binary name and build configuration. Production images should also include required CA certificates and any necessary runtime libraries.

## 6.2 Evaluation Worker Image

The Python evaluation worker should have an independent image.

Requirements:

- Pin Python version.
- Use `uv.lock`.
- Install dependencies reproducibly.
- Run as non-root.
- Separate evaluation dependencies from gateway dependencies.
- Avoid bundling provider secrets.

## 6.3 Image Tagging

Use immutable image references for deployment.

Examples:

```text
llm-gateway:1.0.0
llm-gateway:git-<commit-sha>
llm-gateway-worker:1.0.0
llm-gateway-worker:git-<commit-sha>
```

Avoid using `latest` as the production deployment reference.

# 7. Docker Compose

The initial local environment uses Docker Compose.

Suggested services:

```yaml
services:
  gateway:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    env_file:
      - .env
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped

  evaluation-worker:
    build:
      context: .
      dockerfile: Dockerfile.worker
    env_file:
      - .env
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy
    restart: unless-stopped

  postgres:
    image: postgres:17
    environment:
      POSTGRES_DB: llm_gateway
      POSTGRES_USER: gateway
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U gateway -d llm_gateway"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7
    command: ["redis-server", "--appendonly", "yes"]
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:
```

This is a starting example, not a complete production Compose file.

Before production use:

- Pin container images to reviewed versions or digests.
- Configure Redis authentication and network restrictions.
- Use secret files or an external secret manager.
- Restrict published ports.
- Configure resource limits.
- Configure logging and retention.
- Define backup procedures.
- Configure container health checks and shutdown behavior.

Do not expose PostgreSQL or Redis publicly.

# 8. Configuration Management

Configuration must be separated from application code.

## 8.1 Configuration Categories

| Category | Examples | Storage |
|---|---|---|
| Application | Host, port, environment | Environment/config |
| Database | Connection pool settings | Secret reference + config |
| Redis | Endpoint, pool settings | Secret reference + config |
| Provider | Provider URL, capabilities | Versioned configuration |
| Routing | Strategy, eligibility rules | Versioned configuration |
| Policies | Budgets, restrictions | Database/config |
| Telemetry | Collector endpoint, sampling | Environment/config |
| Secrets | Provider API keys, passwords | Secret manager |

## 8.2 Environment Variables

Example `.env.example`:

```dotenv
APP_ENV=local
APP_HOST=0.0.0.0
APP_PORT=8080

DATABASE_URL=postgres://gateway:change-me@postgres:5432/llm_gateway
REDIS_URL=redis://redis:6379

OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317

LOG_LEVEL=info

GATEWAY_MASTER_KEY_FILE=/run/secrets/gateway_master_key
OPENAI_API_KEY_FILE=/run/secrets/openai_api_key
ANTHROPIC_API_KEY_FILE=/run/secrets/anthropic_api_key
```

The application must explicitly support file-based secret loading if these `_FILE` variables are used.

The example credentials are placeholders, not usable secrets.

Never commit `.env`.

# 9. Secret Management

## 9.1 Development

For local development:

- Use Docker Compose secrets or protected local secret files.
- Keep `.env` out of Git.
- Use dedicated development provider keys.
- Limit provider spending.
- Rotate credentials periodically.

## 9.2 Staging and Production

Use an external secret manager, such as:

- HashiCorp Vault.
- AWS Secrets Manager.
- Azure Key Vault.
- Google Secret Manager.

The gateway uses the existing `SecretProvider` abstraction.

Provider credentials must never appear in:

- Application logs.
- Error messages.
- Debug output.
- Container images.
- Git history.
- Ordinary API responses.
- Telemetry attributes.

Virtual API keys and provider credentials have different storage requirements:

- Virtual API keys are verified using stored cryptographic hashes.
- Provider credentials are retrieved through the secret-management integration.

# 10. Networking

## 10.1 Network Boundaries

Separate:

- Public ingress.
- Application network.
- Infrastructure network.
- Observability network.

Only the reverse proxy should expose public HTTP/HTTPS traffic.

PostgreSQL, Redis, and internal telemetry services must remain private.

## 10.2 Egress Control

The gateway needs controlled outbound access to model providers.

Apply:

- HTTPS enforcement.
- Provider endpoint allowlists where feasible.
- SSRF protections.
- DNS validation and rebinding defenses.
- Restricted redirects.
- Connection and request timeouts.
- Egress monitoring.

Do not allow tenant-controlled provider URLs to bypass SSRF controls.

# 11. Database Deployment and Migrations

PostgreSQL is the authoritative store for:

- Tenants.
- Projects.
- API key hashes.
- Policies.
- Budgets.
- Reservations.
- Usage records.
- Financial ledger.
- Provider and model metadata.
- Audit events.
- Transactional outbox.

## 11.1 Migration Strategy

Use version-controlled migrations.

Requirements:

- Migrations must be repeatable in CI.
- Production migrations must be executed as a controlled deployment job.
- Application startup must not cause uncontrolled schema changes.
- Destructive changes must use an expand-and-contract approach.
- Rollback procedures must be documented.

## 11.2 Expand-and-Contract Example

1. Add a new nullable column.
2. Deploy code that supports both old and new representations.
3. Backfill existing records.
4. Switch reads and writes to the new column.
5. Verify data.
6. Remove the old column in a later release.

Avoid destructive schema changes in the same deployment that first introduces the replacement.

# 12. Redis Deployment

Redis supports operational workloads:

- Distributed rate limiting.
- Concurrency permits.
- Exact caching.
- Short-lived coordination state.

PostgreSQL remains authoritative for financial accounting.

Redis configuration must address:

- Authentication.
- Private networking.
- Persistence requirements.
- Memory limits.
- Eviction policy.
- Availability.
- Recovery.
- Monitoring.

Do not configure eviction in a way that silently removes active concurrency permit state.

Rate-limit and concurrency behavior during Redis failure must follow the policies in `docs/rate-limiting-and-concurrency.md`.

# 13. Health Checks and Graceful Shutdown

## 13.1 Health Endpoints

Expose:

| Endpoint | Purpose |
|---|---|
| `/health/live` | Process liveness |
| `/health/ready` | Readiness to accept traffic |
| `/health/dependencies` | Dependency diagnostics, protected as appropriate |

Liveness should answer whether the process is functioning.

Readiness should reflect whether the instance can safely accept the required workload.

A single provider outage should not necessarily make the gateway unready if eligible alternative providers remain available.

## 13.2 Graceful Shutdown

On termination:

1. Stop accepting new requests.
2. Stop accepting new streaming sessions.
3. Allow in-flight requests a bounded drain period.
4. Cancel requests exceeding the shutdown deadline.
5. Reconcile or persist execution state.
6. Release concurrency permits safely.
7. Flush required telemetry and events within bounded time.
8. Close database and Redis connections.

Never rely exclusively on graceful shutdown for financial correctness. Recovery procedures must handle abrupt process termination.

# 14. Observability Deployment

## 14.1 Components

| Component | Responsibility |
|---|---|
| OpenTelemetry SDK | Instrumentation |
| OpenTelemetry Collector | Telemetry pipeline |
| Prometheus | Metrics |
| Grafana | Dashboards and alerts |
| Tempo or Jaeger | Distributed traces |
| Loki | Log aggregation |
| Langfuse | LLM observability interface |

The exact stack can be simplified for the initial release.

## 14.2 Dashboards

Provide dashboards for:

- Gateway health.
- Request volume.
- Gateway-added latency.
- Provider health.
- Model routing.
- Token usage.
- Cost and budget consumption.
- Rate-limit rejection.
- Concurrency utilization.
- Cache hit rate.
- Evaluation quality.
- Event backlog.

## 14.3 Alerts

Examples:

- Gateway availability below SLO.
- Provider error rate above threshold.
- Budget settlement backlog.
- Unknown provider outcomes increasing.
- Redis unavailable.
- Database connection pool saturation.
- Event consumer lag.
- Evaluation failures.
- Unusual cost increase.
- Authentication abuse.

Telemetry failure must not indefinitely block inference execution.

# 15. Backup and Disaster Recovery

## 15.1 PostgreSQL Backup

Use:

- Scheduled logical backups.
- Physical backups where appropriate.
- WAL archiving for point-in-time recovery when configured.
- Encrypted backup storage.
- Backup retention policies.
- Restore verification.

A backup is not considered reliable until restoration has been tested.

## 15.2 Redis Recovery

Redis persistence and recovery must be selected according to the role of each data structure.

Reconstructible cache entries may be discarded.

Operational state such as active concurrency permits requires a recovery strategy that prevents uncontrolled over-admission after restart.

PostgreSQL remains authoritative for budgets and financial records.

## 15.3 Recovery Objectives

Define measurable recovery objectives before production.

| Metric | Initial Planning Target |
|---|---|
| RPO | 15 minutes |
| RTO | 1 hour |

These are proposed targets, not demonstrated guarantees. Validate them through backup restoration and disaster-recovery exercises.

## 15.4 Recovery Procedure

1. Identify the failure and affected components.
2. Stop unsafe traffic or enter degraded mode.
3. Restore infrastructure dependencies.
4. Recover PostgreSQL to the selected recovery point.
5. Reconcile incomplete executions and financial records.
6. Restore Redis operational state or rebuild it safely.
7. Verify secrets and provider connectivity.
8. Validate readiness and critical workflows.
9. Gradually restore traffic.
10. Record incident findings.

# 16. CI/CD Pipeline

## 16.1 Pull Request Pipeline

```mermaid
flowchart TD
    A["Pull Request"] --> B["Formatting and Lint"]
    B --> C["Unit Tests"]
    C --> D["Integration Tests"]
    D --> E["Contract Tests"]
    E --> F["Security Scans"]
    F --> G["Build Images"]
    G --> H["Image Scans"]
    H --> I["Quality Gate"]
    I -->|Pass| J["Merge"]
    I -->|Fail| K["Reject"]
```

## 16.2 Release Pipeline

```mermaid
flowchart TD
    A["Release Tag"] --> B["Build Immutable Images"]
    B --> C["Generate SBOM"]
    C --> D["Scan Images"]
    D --> E["Publish Images"]
    E --> F["Deploy Staging"]
    F --> G["Run Smoke Tests"]
    G --> H["Run Critical E2E Tests"]
    H --> I{"Release Approved?"}
    I -->|Yes| J["Production Deployment"]
    I -->|No| K["Stop Release"]
    J --> L["Health Verification"]
    L --> M["Monitor and Validate"]
```

## 16.3 Deployment Safety

- Use immutable image tags.
- Record deployed commit SHA.
- Validate database migration compatibility.
- Support controlled rollback.
- Use staged rollout where supported.
- Verify health after deployment.
- Preserve audit records for release operations.

A rollback must account for database compatibility. Reverting an application image does not automatically reverse a database migration.

# 17. Deployment Scaling

## 17.1 Horizontal Gateway Scaling

The gateway should be stateless wherever practical.

Multiple gateway instances may share:

- PostgreSQL.
- Redis.
- Event transport.
- Secret manager.
- Telemetry collector.

Avoid process-local state as the sole source of truth for:

- Rate limits.
- Global concurrency.
- Financial budgets.
- Tenant authorization.
- Execution accounting.

Local connection pools and process-local semaphores can still protect individual instances.

## 17.2 Worker Scaling

Evaluation workers can scale independently.

Use:

- Consumer groups or an equivalent queue-consumer model.
- Idempotent event handling.
- Bounded concurrency.
- Evaluation budgets.
- Dead-letter or failed-event handling.
- Backpressure.

Do not let evaluation workloads consume all resources required by synchronous inference.

# 18. Deployment Security Checklist

- [ ] TLS enabled for external traffic.
- [ ] Internal services not publicly exposed.
- [ ] Secrets retrieved securely.
- [ ] Containers run as non-root.
- [ ] Images scanned.
- [ ] Dependencies audited.
- [ ] PostgreSQL access restricted.
- [ ] Redis access restricted.
- [ ] Network segmentation configured.
- [ ] Provider egress controlled.
- [ ] Logs redact sensitive values.
- [ ] Audit events enabled.
- [ ] Backup encryption enabled.
- [ ] Restore procedure tested.
- [ ] Production credentials separated from staging.

# 19. Operational Runbooks

Create operational runbooks for:

| Runbook | Purpose |
|---|---|
| Provider outage | Diagnose and recover provider connectivity |
| Redis outage | Apply safe degraded-mode behavior |
| PostgreSQL outage | Protect persistent state and recover service |
| Budget reconciliation | Investigate accounting discrepancies |
| Secret rotation | Rotate provider and platform credentials |
| API key compromise | Revoke and replace virtual keys |
| High cost alert | Investigate unexpected spend |
| Gateway overload | Reduce load and restore capacity |
| Evaluation backlog | Recover delayed evaluation processing |
| Disaster recovery | Restore service from backups |

# 20. Deployment Acceptance Criteria

Deployment architecture is ready for the initial release when:

- [ ] Gateway builds into a reproducible Docker image.
- [ ] Evaluation worker builds independently.
- [ ] Local Docker Compose environment starts successfully.
- [ ] PostgreSQL and Redis health checks work.
- [ ] Database migrations execute through a controlled process.
- [ ] Secrets are not embedded in images or source code.
- [ ] Gateway supports graceful shutdown.
- [ ] Readiness and liveness endpoints work.
- [ ] Logs, metrics, and traces are available.
- [ ] Backup and restore procedures are tested.
- [ ] CI/CD release gates are operational.
- [ ] Deployment rollback is documented and validated.
- [ ] Critical end-to-end workflows pass in staging.

# 21. Architectural Decisions

Proposed ADRs:

| ADR | Decision |
|---|---|
| ADR-070 | Docker Compose for initial local deployment |
| ADR-071 | Multi-stage Rust container builds |
| ADR-072 | Separate evaluation worker deployment |
| ADR-073 | External secret manager for production |
| ADR-074 | Controlled database migration jobs |
| ADR-075 | OpenTelemetry-based deployment observability |
| ADR-076 | PostgreSQL backup and recovery strategy |
| ADR-077 | Stateless gateway instances |
| ADR-078 | Kubernetes deferred until operational requirements justify it |

Reconcile identifiers with the existing ADR index.

# 22. Related Documents

- `docs/product.md`
- `docs/architecture.md`
- `docs/security.md`
- `docs/provider-integration.md`
- `docs/reliability.md`
- `docs/observability.md`
- `docs/budget-and-usage-accounting.md`
- `docs/rate-limiting-and-concurrency.md`
- `docs/test-strategy.md`
- `docs/definition-of-done.md`
- `docs/build-plan.md`


