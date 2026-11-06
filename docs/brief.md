# BioViva - Project Brief

## Executive Summary

BioViva is a high-performance, secure e-commerce platform specializing in premium organic, bio-certified products. The platform delivers transparency in bio-certification and artisanal origin through full batch traceability (origin, harvest date, acidity/purity metrics) while enabling seamless digital purchasing. Built with a Domain-Driven Design (DDD) and Clean Code Architecture using Rust, Tokio, and Axum, BioViva connects health-conscious consumers with verified organic artisanal products via a PostgreSQL persistence layer with explicit schemas (`auth`, `catalog`, `sales`, `inventory`) and JWT/Argon2 authentication.

## Objectives & Success Metrics

| Objective | Success Metric |
|---|---|
| Launch MVP e-commerce platform with organic product catalog | 100% of core features functional at launch |
| Enable full batch traceability for all products | 100% of products have origin/harvest metadata captured |
| Secure authentication and identity management | <0.1% authentication failure rate; JWT sessions secure |
| Support artisanal producer onboarding and batch management | Producers can create/manage batches within 5 minutes |
| Operate with fixed EUR currency for MVP | No currency conversion edge cases in production |

## Target Audience

| Persona | Needs & Pain Points |
|---|---|
| **B2C Consumers** (Health-conscious shoppers) | Verify organic certification; trace product origin; secure payment; fast delivery |
| **Artisanal Producers/Suppliers** | Manage batch metadata (origin, harvest date, acidity, purity); list products; view order flow |
| **Platform Administrators** | Catalog management; inventory oversight; order fulfillment; user/role management |

## Functional Requirements (Epics)

1. **Authentication & Identity Management**
   - User registration/login with email and password
   - JWT-based authentication with Argon2 password hashing
   - Role-based access control (consumer, producer, admin)
   - Password reset flows

2. **Product Catalog with Organic Certification Attributes & Custom Variants**
   - CRUD operations for products
   - Certification attributes: organic standard, certifying body, certificate number
   - Custom variants: weight, size, origin region, harvest date
   - Product search and filtering by certification, attributes

3. **Order Processing & Cart Management**
   - Shopping cart with persistent session storage
   - Order creation, validation, and status tracking
   - Order history per user
   - Order cancellation/refund flow (basic)

4. **Basic Stock & Origin Batch Tracking**
   - Batch creation and assignment to products
   - Origin tracking: source farm/region, harvest date
   - Purity and acidity metrics capture and display
   - Inventory decrement on order fulfillment

## Non-Functional Requirements

| Category | Requirement |
|---|---|
| **Performance** | API response < 200ms for catalog queries under normal load |
| **Security** | JWT validation; Argon2 password hashing; HTTPS-only; SQL injection prevention via parameterized queries |
| **Reliability** | 99.9% uptime; graceful degradation on DB connectivity loss |
| **Scalability** | Horizontal scaling via stateless Axum services + Tokio; single PostgreSQL instance with schema partitioning |
| **Maintainability** | DDD boundaries (Domain, Application, Infrastructure crates); clean code standards; comprehensive test coverage |
| **Observability** | Structured logging (tracing); metrics endpoints; health checks |

## Technical Boundary Constraints

- **Language**: Rust exclusively (API, Domain, Infrastructure crates)
- **Framework**: Axum for HTTP, Tokio for async runtime
- **Persistence**: PostgreSQL single shared instance
  - Schemas: `auth`, `catalog`, `sales`, `inventory`
  - No cross-schema joins beyond necessary read models
- **Authentication**: Integrated Auth Service using JWT and Argon2 only
- **Currency**: Fixed EUR for MVP; no multi-currency support
- **Architecture**: DDD with Clean Code Architecture; separation of Domain (pure business logic), Application (use cases), Infrastructure (DB, external services)
- **Deployment**: Containerized Rust services; PostgreSQL as managed or self-hosted instance

## Risks & Assumptions

| Risk | Mitigation |
|---|---|
| **Single PostgreSQL bottleneck** | Implement read replicas eventually; optimize queries within schema boundaries |
| **JWT/Argon2 integration complexity** | Use established crates (jsonwebtoken, argon2); thorough security testing |
| **Producer onboarding adoption** | Provide simple UI flow; batch validation rules; support workflow |
| **Batch traceability data completeness** | Require minimal mandatory fields at launch; expand metadata over time |
| **Rust async learning curve** | Team familiarization spike; adopt idiomatic Tokio patterns early |

*Assumptions: Development team has intermediate Rust experience; PostgreSQL 13+ available; EUR as sole transaction currency for MVP timeline.*