# Rustonis — Product Requirements Document

> Version: 0.1 | Date: 2026-02-28 | Statut: Draft

---

## 1. Vision

**Rustonis est AdonisJS pour Rust.**

Quand un développeur JavaScript maîtrisant AdonisJS (ou NestJS, Laravel) passe à Rust,
Rustonis lui offre les mêmes repères philosophiques, la même productivité, et ajoute
ce qu'AdonisJS n'a pas encore.

**Tagline :** *"The AdonisJS of Rust. Familiar architecture, zero compromises."*

---

## 2. Problème

La phrase *"Rust needs a web framework for lazy developers"* (Nicole Tietz, 2024) résume
le gap du marché : Rust a les performances et la sûreté, mais aucun framework ne donne
la productivité d'AdonisJS/Rails/Django.

**Loco.rs** est le seul concurrent batteries-included, mais il cible les développeurs
**Rails/Ruby** — pas l'univers JavaScript/AdonisJS. Rustonis occupe ce segment non couvert.

### Chiffres clés

- ~50 000 développeurs AdonisJS actifs
- ~2 millions de développeurs Node.js backend
- ~1.5 million de développeurs Rust (croissance +40% /an)
- 0 framework ciblant explicitement la migration AdonisJS → Rust

---

## 3. Cibles

| Segment | Description | Priorité |
|---------|-------------|----------|
| **Primaire** | Devs AdonisJS qui veulent migrer vers Rust | P0 |
| **Secondaire** | Devs NestJS/Express cherchant un framework Rust productif | P1 |
| **Tertiaire** | Devs Rust voulant de la productivité sans sacrifier le contrôle | P2 |

### User Story Principal

> **As a** développeur JavaScript/TypeScript (AdonisJS, NestJS, Express)
> **I want** retrouver mes repères de productivité quand je passe à Rust
> **So that** je puisse bénéficier des garanties Rust (safety, performance) sans
> sacrifier 6 mois de productivité

---

## 4. Les 4 Piliers Philosophiques

Identiques à AdonisJS, traduits pour Rust :

| Pilier | Description |
|--------|-------------|
| **Stable over Trendy** | Pas de chasing des crates du moment, SemVer strict |
| **Cohesive over Flexible** | Un ORM, un validator, un auth system — tout intégré |
| **Productive over Minimal** | Tout inclus + macros qui éliminent le boilerplate Rust |
| **Opinionated by Design** | Structure imposée, zéro débat architectural |

---

## 5. Analyse Concurrentielle

| Framework | Philosophie | DX | Batteries | Cible |
|-----------|-------------|-----|-----------|-------|
| Actix-web | Performance pure | ⭐⭐⭐ | ❌ | Perf-first |
| Axum | Ergonomique, modulaire | ⭐⭐⭐⭐ | ❌ | Rust natives |
| Rocket | Type-safe, expressif | ⭐⭐⭐⭐⭐ | ⭐⭐ | Débutants Rust |
| Loco.rs | Rails for Rust | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Ruby/Rails devs |
| Kit.rs | Laravel-inspired | ⭐⭐⭐ | ⭐⭐ | Laravel/PHP devs |
| **Rustonis** | AdonisJS for Rust | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | **JS/AdonisJS devs** |

---

## 6. Architecture

### Décision Architecturale : Variante C retenue

Rustonis utilise **Axum comme HTTP engine** mais construit un **IoC Container souverain**
et mise tout sur le **CLI first** (Rustonis CLI = Ace).

**Justification :**
- L'IoC Container est le cœur philosophique d'AdonisJS — il faut en avoir un souverain
- Axum est la fondation la moins opinionated → contraintes minimales imposées
- Le CLI est le premier contact développeur → investir tôt crée l'effet "wow"

### Mapping AdonisJS → Rustonis

| Composant AdonisJS | Crate Rustonis | Phase |
|-------------------|----------------|-------|
| IoC Container | `rustonis-container` | v0.1 |
| Service Providers | macro `#[provider]` | v0.1 |
| Routing (`start/routes.ts`) | `start/routes.rs` | v0.1 |
| Controllers | macro `#[controller]` | v0.1 |
| Middleware | `start/kernel.rs` | v0.1 |
| VineJS Validation | `rustonis-validator` | v0.2 |
| Config system | `rustonis-config` | v0.1 |
| Lucid ORM | `rustonis-orm` | v0.4 |
| Auth Guards | `rustonis-auth` | v0.5 |
| Mailer | `rustonis-mailer` | v0.5 |
| Edge Templates | `rustonis-views` | v0.5 |
| Queues | `rustonis-queue` | v0.7 |
| WebSockets | `rustonis-ws` | v0.7 |
| Cache | `rustonis-cache` | v0.8 |
| SSE | `rustonis-sse` | v0.9 |
| Ace CLI | `rustonis-cli` | v0.1 |

### Structure de projet générée par `rustonis new`

```
my-app/
├── app/
│   ├── controllers/
│   ├── models/
│   ├── middleware/
│   ├── validators/
│   ├── services/
│   ├── mailers/
│   └── jobs/
├── config/
│   ├── app.rs
│   ├── database.rs
│   ├── auth.rs
│   └── mail.rs
├── database/
│   ├── migrations/
│   └── seeders/
├── providers/
├── resources/
│   └── views/
├── start/
│   ├── routes.rs
│   ├── kernel.rs
│   └── events.rs
├── tests/
├── .env
├── .env.example
└── Cargo.toml
```

---

## 7. Composants Clés

### 7.1 IoC Container

```rust
#[provider]
pub struct DatabaseProvider;

#[async_trait]
impl ServiceProvider for DatabaseProvider {
    async fn register(&self, container: &mut Container) {
        container.bind_singleton::<Database>(|| async {
            Database::connect(env!("DATABASE_URL")).await.unwrap()
        });
    }

    async fn boot(&self, container: &Container) {
        let db = container.make::<Database>().await;
        db.run_migrations().await.unwrap();
    }
}
```

### 7.2 Controllers avec DI

```rust
#[controller]
pub struct UserController {
    #[inject] db: Arc<Database>,
    #[inject] mailer: Arc<Mailer>,
}

impl UserController {
    pub async fn store(
        &self,
        req: Valid<CreateUserDto>,
    ) -> JsonResponse<UserDto> {
        let user = User::create(req.into_inner()).exec(&self.db).await?;
        self.mailer.send(WelcomeMail::new(&user)).await?;
        JsonResponse::created(UserDto::from(user))
    }
}
```

### 7.3 Routing

```rust
// start/routes.rs
pub fn register(router: &mut Router) {
    router.get("/", HomeController::index);

    router.group()
        .prefix("/api/v1")
        .middleware(AuthMiddleware::guard("api"))
        .routes(|r| {
            r.resource("/users", UserController);
            r.get("/profile", UserController::show);
        });
}
```

### 7.4 ORM (Active Record style)

```rust
#[model]
pub struct User {
    #[column(primary_key, auto_increment)]
    pub id: i64,
    pub email: String,
    pub password: String,
    pub created_at: DateTime<Utc>,

    #[has_many(Post)]
    pub posts: HasMany<Post>,
}

// Query Builder
let users = User::query()
    .where_clause("is_active", true)
    .preload("posts")
    .order_by("created_at", "desc")
    .paginate(page, 20)
    .exec()
    .await?;
```

### 7.5 Validator

```rust
#[validator]
pub struct CreateUserValidator;

impl Validator for CreateUserValidator {
    type Schema = CreateUserDto;

    fn schema() -> Schema<CreateUserDto> {
        schema! {
            email: vine().string().email().normalize(),
            password: vine().string().min_length(8).confirmed(),
            name: vine().string().min_length(2).max_length(100),
        }
    }
}
```

### 7.6 Auth Guards

```rust
// config/auth.rs
pub fn auth_config() -> AuthConfig {
    AuthConfig::new()
        .guard("web", SessionGuard::new(UserProvider::db()))
        .guard("api", BearerTokenGuard::new(TokenProvider::opaque()))
        .guard("jwt", JwtGuard::new(UserProvider::db()))
}
```

---

## 8. CLI : rustonis (équivalent Ace)

```bash
# Projet
rustonis new my-app [--api | --fullstack | --microservice]
rustonis new:module auth

# Génération de code
rustonis make:controller User
rustonis make:model User --migration
rustonis make:middleware AuthMiddleware
rustonis make:provider CacheProvider
rustonis make:mailer WelcomeMail
rustonis make:command SendNewsletterCommand
rustonis make:validator CreateUserValidator
rustonis make:job ProcessPaymentJob
rustonis make:listener OrderCreatedListener

# Base de données
rustonis db:migrate
rustonis db:rollback
rustonis db:seed
rustonis db:fresh
rustonis db:studio

# Développement
rustonis serve
rustonis serve --watch
rustonis repl

# Production
rustonis build
rustonis start
```

---

## 9. Ce que Rustonis ajoute vs AdonisJS

Basé sur les issues GitHub AdonisJS les plus votées et le feedback communautaire :

| Feature | Justification |
|---------|---------------|
| **Message Bus first-class** (Kafka, RabbitMQ, NATS) | Issue #1 demandée sur le GitHub AdonisJS |
| **Hot Reload vrai** | Gap critique de tout l'écosystème Rust |
| **Observable by default** (OpenTelemetry) | AdonisJS v7 y travaille ; Rustonis l'intègre dès v1 |
| **Admin Panel Generator** (`make:admin`) | Inexistant dans AdonisJS |
| **API Client TS Generator** (`generate:client`) | Supérieur à Tuyau d'Adonis |
| **Deployment Toolkit** (Docker, Fly.io, Railway) | Gap de tout l'écosystème Rust |

---

## 10. ADRs (Architecture Decision Records)

### ADR-001 : Axum comme HTTP Engine

**Statut :** Accepté

**Décision :** Utiliser Axum comme couche HTTP. Rustonis gère IoC, ORM, Auth, CLI.

**Conséquences positives :** Tower ecosystem, performance prouvée, Tokio natif.

**Mitigation du risque :** Isolation via `rustonis-http` pour découpler des breaking changes Axum.

### ADR-002 : Proc-Macros + Builder Pattern

**Statut :** Accepté

**Décision :** Proc-macros (`#[controller]`, `#[inject]`, `#[model]`) pour la syntaxe propre,
Builder Pattern comme fallback explicite pour les devs Rust.

**Justification :** Les macros réduisent le choc culturel JS → Rust. Le Builder reste pour
les développeurs qui préfèrent le code explicite.

### ADR-003 : IoC Container Souverain

**Statut :** Accepté

**Décision :** Container implémenté par Rustonis, pas délégué à Axum ou autre crate.

**Justification :** C'est le cœur philosophique d'AdonisJS. Le déléguer = perdre l'identité
du framework.

**Inspiration :** Crates `shaku` et `inject` pour l'implémentation technique.

---

## 11. Risques

| Risque | Probabilité | Impact | Mitigation |
|--------|-------------|--------|------------|
| IoC Container trop complexe en Rust | Élevée | Élevé | POC d'abord, phases progressives, s'inspirer de `shaku` |
| Proc-macros instables entre versions Rust | Moyenne | Élevé | CI sur multiple toolchain versions |
| Loco.rs cible aussi les JS devs en v2 | Moyenne | Moyen | Différenciation explicite : philosophie AdonisJS, CLI supérieur |
| Performance dégradée vs Axum bare metal | Faible | Faible | Benchmarks publics, overhead documenté |
| Communauté trop petite pour maintenir | Moyenne | Élevé | Docs excellents dès le début, good first issues |

---

## 12. Métriques de Succès

| Métrique | 6 mois | 1 an | 2 ans |
|----------|--------|------|-------|
| GitHub Stars | 500 | 2 000 | 5 000 |
| Contributeurs actifs | 5 | 20 | 50 |
| Apps en production | 1 (démo officielle) | 10 | 50 |
| Crates publiées | 5 | 15 | 30 |
| Pages de documentation | 20 | 50 | 100 |
| Membres Discord | 50 | 300 | 1 000 |

---

## Références

- [AdonisJS Philosophy](https://adonisjs.com/about)
- [AdonisJS v7 Roadmap](https://adonisjs.com/blog/roadmap-to-adonisjs-7)
- [Loco.rs](https://loco.rs) — concurrent principal
- [Rust needs a web framework for lazy developers](https://ntietz.com/blog/rust-needs-a-web-framework-for-lazy-developers/)
- [Rust web framework comparison](https://github.com/flosse/rust-web-framework-comparison)
