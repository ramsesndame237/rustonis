# Rustonis — Roadmap & Organisation des Tâches

> Dernière mise à jour : 2026-02-28
> v0.3 HTTP & Routing — ✅ COMPLÉTÉ

---

## Phases de Développement

```
v0.1-0.3  Foundation       ✅ Complété
v0.4      Validation        ✅ Complété
v0.5-0.6  Core Features    ← On est ici
v0.7-1.0  Advanced
v1.x      Ecosystem
```

---

## Phase 1 — Foundation (v0.1 → v0.3)

**Objectif :** Un développeur peut créer un projet avec `rustonis new`, démarrer
un serveur, définir des routes, injecter des dépendances.

**Critère de succès :** "Hello World" avec IoC Container fonctionnel en 5 minutes.

### Milestone v0.1 — Squelette & CLI

- [x] **CLI-001** — `rustonis new <name>` génère la structure AdonisJS-like complète
- [x] **CLI-002** — `rustonis serve` démarre le serveur de dev
- [x] **CLI-003** — `rustonis serve --watch` avec hot reload (cargo-watch)
- [x] **CORE-001** — `rustonis-core` transformé de binaire en lib crate
- [x] **CORE-002** — Structure de workspace propre (cli / core / macros)
- [x] **CORE-003** — Système de configuration typé (`.env` → structs Rust)

### Milestone v0.2 — IoC Container

> Composant le plus critique et différenciant du framework.

- [x] **IOC-001** — `Container` struct avec `bind_singleton<T>` et `bind_transient<T>`
- [x] **IOC-002** — `container.make::<T>()` pour résoudre une dépendance
- [x] **IOC-003** — `Application` struct qui orchestre les providers
- [x] **IOC-004** — Trait `ServiceProvider` avec `register()` et `boot()`
- [ ] **IOC-005** — Macro proc `#[provider]` — code generation du boilerplate
- [ ] **IOC-006** — Macro proc `#[inject]` — injection automatique dans les structs
- [x] **IOC-007** — Tests unitaires du container (singleton, transient, résolution cyclique)

### Milestone v0.3 — HTTP & Routing ✅

- [x] **HTTP-001** — Intégration Axum comme HTTP engine (isolation via `rustonis-http`)
- [x] **HTTP-002** — `Router` avec méthodes `get`, `post`, `put`, `patch`, `delete`
- [x] **HTTP-003** — Route groups avec `group(prefix, callback)` et `merge()`
- [x] **HTTP-004** — `start/routes.rs` — point d'entrée des routes comme dans Adonis
- [x] **HTTP-005** — `start/kernel.rs` — registration des middleware globaux via `HttpServer::layer()`
- [x] **HTTP-006** — Placeholder `#[controller]` — macro proc à compléter en v0.4+
- [x] **HTTP-007** — `JsonResponse<T>` et `NoContent` — types HTTP typés
- [x] **HTTP-008** — Gestion d'erreurs HTTP unifiée (`AppError` → HTTP response)
- [x] **HTTP-009** — `rustonis make controller <Name> [--resource]` — générateur CLI

---

## Phase 2 — Core Features (v0.4 → v0.6)

**Objectif :** Une vraie app CRUD avec auth, base de données et emails est possible.

### Milestone v0.4 — Validation ✅

- [x] **VAL-001** — `rustonis-validator` — crate de validation
- [x] **VAL-002** — `#[derive(Validate)]` avec attributs de champ (style VineJS)
- [x] **VAL-003** — Type `Valid<T>` — extracteur Axum qui valide automatiquement
- [x] **VAL-004** — Règles built-in : required, email, url, alphanumeric, min_length, max_length, min, max, confirmed, one_of
- [x] **VAL-005** — Messages d'erreur customisables via `message = "..."`
- [x] **VAL-006** — `rustonis make validator <Name>` — générateur CLI

### Milestone v0.5 — ORM

- [ ] **ORM-001** — `rustonis-orm` — crate ORM basée sur SQLx
- [ ] **ORM-002** — Macro `#[model]` — génère le boilerplate Active Record
- [ ] **ORM-003** — Query builder fluent : `where_clause`, `order_by`, `paginate`
- [ ] **ORM-004** — Relations : `#[has_many]`, `#[belongs_to]`, `#[has_one]`
- [ ] **ORM-005** — Preloading / eager loading des relations
- [ ] **ORM-006** — Support multi-DB : PostgreSQL, MySQL, SQLite
- [ ] **ORM-007** — Système de migrations (`rustonis db:migrate`, `db:rollback`)
- [ ] **ORM-008** — Seeds (`rustonis db:seed`, `db:fresh`)
- [ ] **ORM-009** — `rustonis make:model <Name> --migration` — générateur CLI

### Milestone v0.6 — Auth, Mailer, Views

- [ ] **AUTH-001** — `rustonis-auth` — système de guards
- [ ] **AUTH-002** — `SessionGuard` — auth par session
- [ ] **AUTH-003** — `BearerTokenGuard` — auth par token opaque
- [ ] **AUTH-004** — `JwtGuard` — auth JWT
- [ ] **AUTH-005** — `AuthMiddleware::guard("api")` — middleware de protection des routes
- [ ] **AUTH-006** — Hash des mots de passe (argon2)
- [ ] **MAIL-001** — `rustonis-mailer` — système d'envoi d'emails
- [ ] **MAIL-002** — Trait `Mailable` pour définir un mail
- [ ] **MAIL-003** — Templates d'email (Tera)
- [ ] **MAIL-004** — Drivers : SMTP, SES, Mailgun
- [ ] **MAIL-005** — `rustonis make:mailer <Name>` — générateur CLI
- [ ] **VIEW-001** — `rustonis-views` — moteur de templates (Tera, équivalent Edge)
- [ ] **VIEW-002** — `rustonis make:view <name>` — générateur CLI

---

## Phase 3 — Advanced Features (v0.7 → v1.0)

**Objectif :** Production-ready + features qu'AdonisJS n'a pas encore.

### Milestone v0.7 — Queues & WebSockets

- [ ] **QUEUE-001** — `rustonis-queue` — système de jobs asynchrones
- [ ] **QUEUE-002** — Drivers : Redis, DB (SQLite/Postgres)
- [ ] **QUEUE-003** — `rustonis make:job <Name>` — générateur CLI
- [ ] **WS-001** — `rustonis-ws` — WebSockets avec architecture par channels
- [ ] **WS-002** — Auth middleware pour les channels WS

### Milestone v0.8 — Cache & Rate Limiting

- [ ] **CACHE-001** — `rustonis-cache` — drivers mémoire et Redis
- [ ] **RATE-001** — Rate limiting intégré sur les routes

### Milestone v0.9 — Observabilité & Hot Reload

- [ ] **OBS-001** — OpenTelemetry intégré au boot (traces, metrics, logs)
- [ ] **OBS-002** — Dashboard dev `/rustonis-devtools` en mode développement
- [ ] **HOT-001** — Hot reload amélioré (vrai HMR, pas juste restart)
- [ ] **SSE-001** — `rustonis-sse` — Server-Sent Events (équivalent Transmit)

### Milestone v1.0 — Stabilisation

- [ ] **DOC-001** — Documentation complète (guides + API reference)
- [ ] **TEST-001** — `rustonis-testing` — helpers de test (équivalent Japa)
- [ ] **BENCH-001** — Benchmarks publics vs Axum bare metal, vs Loco
- [ ] **DEMO-001** — Application de démonstration officielle (blog ou SaaS simple)

---

## Phase 4 — Ecosystem (v1.x)

**Objectif :** Dominer le segment JS devs → Rust. Features différenciantes vs AdonisJS.

- [ ] **BUS-001** — `rustonis-bus` — Message Bus (Kafka, RabbitMQ, NATS)
- [ ] **BUS-002** — `rustonis make:listener <Name>` pour les consumers
- [ ] **ADMIN-001** — Admin Panel Generator (`rustonis make:admin <Model>`)
- [ ] **CLIENT-001** — API Client TypeScript Generator (`rustonis generate:client`)
- [ ] **DEPLOY-001** — `rustonis deploy:docker` — Dockerfile optimisé
- [ ] **DEPLOY-002** — `rustonis deploy:fly` — Deploy Fly.io
- [ ] **DEPLOY-003** — `rustonis deploy:railway` — Deploy Railway
- [ ] **REPL-001** — `rustonis repl` — REPL avec contexte app chargé
- [ ] **STUDIO-001** — `rustonis db:studio` — GUI web pour la DB

---

## Prochaines Actions Immédiates

### Cette semaine — POC IoC Container

La décision architecturale la plus risquée est l'IoC Container en Rust (ownership +
lifetimes + async). Valider la faisabilité avant tout autre développement.

```
1. [ ] Lire les crates : shaku, inject, anymap
2. [ ] Implémenter Container minimal (bind_singleton + make::<T>())
3. [ ] Tester avec 2 providers qui se dépendent
4. [ ] Décider : Container basé sur TypeId ou string keys
```

### Cette semaine — `rustonis new` amélioré

Le CLI actuel génère seulement Cargo.toml + main.rs. L'améliorer pour générer
la structure complète AdonisJS-like.

```
1. [ ] Définir les templates de fichiers (routes.rs, kernel.rs, etc.)
2. [ ] Générer la structure complète avec --api et --fullstack
3. [ ] Initialiser git + .env.example
```

---

## Organisation des Issues GitHub (Labels suggérés)

```
area/cli          → Tout ce qui touche rustonis-cli
area/core         → rustonis-core, IoC Container
area/orm          → rustonis-orm
area/auth         → rustonis-auth
area/validator    → rustonis-validator
area/mailer       → rustonis-mailer
area/queue        → rustonis-queue
area/ws           → rustonis-ws
area/docs         → Documentation

type/feature      → Nouvelle feature
type/bug          → Bug fix
type/refactor     → Refactoring
type/test         → Tests

priority/p0       → Bloquant
priority/p1       → Important
priority/p2       → Nice to have

good-first-issue  → Pour les nouveaux contributeurs
```
