# Resume Vizor Backend

Бэкенд на `Rust + Axum` для платформы-посредника проверки дипломов.

Платформа позволяет:

- вузу загружать дипломы по одному или массово через `CSV/XLSX`
- хранить чувствительные данные дипломов в хешированном виде
- автоматически подписывать записи дипломов цифровой подписью вуза
- студенту находить и привязывать свои дипломы
- студенту делиться временной ссылкой на диплом
- HR/ATS-системам проверять диплом через API
- вузу аннулировать диплом и восстанавливать его обратно

Текущий стек:

- `Rust`
- `Axum`
- `Tokio`
- `SQLx`
- `PostgreSQL`
- `Redis`

`in-memory` persistence сохранен для тестов и быстрых unit/integration-style сценариев.
`Redis` используется там, где он дает максимальный операционный профит: распределенный rate limiting для HR automation endpoint в multi-instance deployment.

## Содержание

- [1. Что умеет система](#1-что-умеет-система)
- [2. Роли в системе](#2-роли-в-системе)
- [3. Архитектура](#3-архитектура)
- [4. Безопасность и модель данных](#4-безопасность-и-модель-данных)
- [5. Структура проекта](#5-структура-проекта)
- [6. Требования](#6-требования)
- [7. Переменные окружения](#7-переменные-окружения)
- [8. Запуск проекта](#8-запуск-проекта)
- [9. Тесты](#9-тесты)
- [10. Общие правила API](#10-общие-правила-api)
- [11. Полное описание API](#11-полное-описание-api)
- [12. Observability и эксплуатация](#12-observability-и-эксплуатация)
- [13. Формат массовой загрузки реестра](#13-формат-массовой-загрузки-реестра)
- [14. Архитектурные потоки](#14-архитектурные-потоки)
- [15. База данных](#15-база-данных)
- [16. Что еще не реализовано](#16-что-еще-не-реализовано)

## 1. Что умеет система

### ВУЗ

- регистрирует диплом по одному
- загружает реестр дипломов массово
- указывает номер студенческого билета при создании каждой записи
- получает для каждой записи:
  - `record_hash`
  - `university_signature`
- может перевести диплом в статус `revoked`
- может вернуть диплом в статус `active`

### Студент

- регистрируется с номером студенческого билета
- ищет свои дипломы
- диплом автоматически привязывается к студенту, если совпали:
  - ФИО из student-аккаунта
  - номер студенческого билета из student-аккаунта
  - данные в записи диплома
- может сгенерировать временную ссылку только на свой диплом

### HR

- проверяет диплом по базовому verify endpoint
- ищет дипломы по номеру диплома или коду вуза
- может использовать отдельный automation endpoint при парсинге резюме
- выпускает и отзывает integration API keys для machine-to-machine интеграций
- при создании ключа выбирает scope: `ats_only`, `hr_automation_only` или `combined`
- scope ключа определяет, какие endpoints доступны и какой burst-limit назначается ключу
- у каждого ключа есть и `daily quota`, и `burst quota`
- дневная квота сейчас задается на уровне кода: каждый integration API key по умолчанию получает `1000` запросов в день
- `POST /api/v1/ats/verify` и `POST /api/v1/hr/automation/verify` работают через `x-api-key`, а не через пользовательский JWT

## 2. Роли в системе

Система знает 3 роли:

- `university`
- `student`
- `hr`

Role-aware маршруты проверяются через заголовок:

```http
role: university
```

или

```http
role: student
```

или

```http
role: hr
```

Также требуется `Authorization: Bearer <jwt>`.

Исключения:

- `/health` не требует авторизации
- `/api/v1/public/diplomas/access/{token}` не требует авторизации
- `/api/v1/auth/register`
- `/api/v1/auth/login`

Маршруты:

- `/api/v1/auth/me`
- `/api/v1/auth/change-password`

требуют `Bearer` токен, но не требуют `role` заголовок.

## 3. Архитектура

Проект разделен на слои:

- `domain`
  - чистые доменные сущности и value objects
- `application`
  - DTO, порты, сервисы, бизнес-логика
- `http`
  - Axum handlers, router, middleware
- `infrastructure`
  - JWT, password hashing, diploma hashing, digital signing, persistence, rate limiting

### Схема слоев

```mermaid
flowchart LR
    A["HTTP / Axum"] --> B["Application Services"]
    B --> C["Domain"]
    B --> D["Application Ports"]
    D --> E["Infrastructure: Postgres Repository"]
    D --> F["Infrastructure: InMemory Repository"]
    B --> G["Infrastructure: JWT / Hashing / Signing / Redis Rate Limit"]
```

### Почему так

- бизнес-логика не зависит от конкретной базы
- `PostgresAppRepository` реализует те же trait-ы, что и `InMemoryAppRepository`
- тесты идут через `in-memory`, а приложение по умолчанию работает через PostgreSQL

## 4. Безопасность и модель данных

### 4.1 Хранение данных диплома

Система не хранит критичные поля диплома в raw-виде для поискового сценария платформы.

Вместо этого для диплома сохраняются хеши:

- `university_code_hash`
- `student_full_name_hash`
- `student_number_hash`
- `student_birth_date_hash`
- `diploma_number_hash`
- `verification_lookup_hash`
- `degree_hash`
- `program_hash`
- `graduation_date_hash`
- `honors_hash`
- `canonical_document_hash`

Хеширование реализовано в:

- [hashing.rs](/D:/Programming/Resume-visitor/src/infrastructure/hashing.rs)

### 4.2 Цифровая подпись записи

Для каждой записи диплома платформа формирует:

- `record_hash`
- `university_signature`

Подпись строится через `Ed25519`.

Важно:

- подпись уникальна для каждого вуза
- приватный ключ конкретного вуза детерминированно выводится из:
  - `UNIVERSITY_SIGNING_KEY`
  - `university_id`

Это реализовано в:

- [signing.rs](/D:/Programming/Resume-visitor/src/infrastructure/signing.rs)

### 4.3 Пароли

- хешируются через `Argon2`
- raw password не сохраняется

Реализация:

- [auth.rs](/D:/Programming/Resume-visitor/src/infrastructure/auth.rs)

### 4.4 JWT

Используются 2 типа signed token:

- обычный access token для авторизации пользователей
- diploma access token для временных публичных ссылок студента

### 4.5 Временная ссылка на диплом

Временная ссылка:

- подписана через JWT secret
- содержит `diploma_id`
- живет ограниченное время
- TTL задается через `DIPLOMA_LINK_TTL_MINUTES`

## 5. Структура проекта

```text
src/
  app.rs
  config.rs
  error.rs
  main.rs
  application/
    dto.rs
    mod.rs
    ports.rs
    services.rs
  domain/
    diploma.rs
    hashing.rs
    ids.rs
    mod.rs
    university.rs
    user.rs
  http/
    auth.rs
    common.rs
    hr.rs
    middleware.rs
    mod.rs
    router.rs
    student.rs
    university.rs
  infrastructure/
    auth.rs
    hashing.rs
    mod.rs
    rate_limit.rs
    signing.rs
    persistence/
      in_memory.rs
      mod.rs
      postgres.rs
migrations/
  0001_init.sql
```

Ключевые файлы:

- [app.rs](/D:/Programming/Resume-visitor/src/app.rs) — bootstrap приложения
- [config.rs](/D:/Programming/Resume-visitor/src/config.rs) — конфигурация из env
- [services.rs](/D:/Programming/Resume-visitor/src/application/services.rs) — основная бизнес-логика
- [router.rs](/D:/Programming/Resume-visitor/src/http/router.rs) — все маршруты
- [postgres.rs](/D:/Programming/Resume-visitor/src/infrastructure/persistence/postgres.rs) — PostgreSQL persistence
- [in_memory.rs](/D:/Programming/Resume-visitor/src/infrastructure/persistence/in_memory.rs) — тестовое storage
- [0001_init.sql](/D:/Programming/Resume-visitor/migrations/0001_init.sql) — миграция базы

## 6. Требования

- Rust toolchain
- Cargo
- PostgreSQL 15+ или совместимая версия

Проверка локально:

```powershell
cargo --version
```

## 7. Переменные окружения

Создайте `.env` на основе `.env.example`.

### Полный список

| Переменная | Обязательна | Пример | Назначение |
|---|---|---|---|
| `APP_PORT` | да | `8080` | HTTP порт приложения |
| `APP_BASE_URL` | да | `http://localhost:8080` | базовый URL, используется для генерации share-link |
| `RUST_LOG` | нет | `info,tower_http=info` | уровень логирования |
| `HR_API_RATE_LIMIT_REQUESTS` | да | `60` | сколько запросов разрешено в automation endpoint за окно |
| `HR_API_RATE_LIMIT_WINDOW_SECONDS` | да | `60` | длина окна rate limit |
| `INTEGRATION_API_KEY_BURST_WINDOW_SECONDS` | да | `10` | окно burst limiter для защиты от серии запросов подряд |
| `INTEGRATION_API_KEY_ATS_ONLY_BURST_LIMIT` | да | `30` | сколько запросов можно сделать за burst-window ключом `ats_only` |
| `INTEGRATION_API_KEY_HR_AUTOMATION_ONLY_BURST_LIMIT` | да | `20` | сколько запросов можно сделать за burst-window ключом `hr_automation_only` |
| `INTEGRATION_API_KEY_COMBINED_BURST_LIMIT` | да | `40` | сколько запросов можно сделать за burst-window ключом `combined` |
| `DATABASE_URL` | да | `postgres://postgres:postgres@localhost:5432/resume_vizor` | строка подключения к PostgreSQL |
| `DATABASE_MAX_CONNECTIONS` | да | `10` | размер пула подключений |
| `REDIS_URL` | нет | `redis://localhost:6379` | Redis для распределенного rate limit на `/api/v1/hr/automation/verify`; если не задан, используется `in-memory` fallback |
| `REDIS_RATE_LIMIT_PREFIX` | нет | `resume_vizor:hr_rate_limit` | префикс ключей rate limiter-а в Redis |
| `DIPLOMA_HASH_KEY` | да | `replace-with-a-long-random-secret` | секрет для keyed hashing полей диплома |
| `JWT_SECRET` | да | `replace-with-another-long-random-secret` | секрет для access token и share token |
| `ATS_API_KEY_SECRET` | да | `replace-with-ats-api-key-secret` | секрет для генерации и хеширования ATS API keys |
| `JWT_TTL_MINUTES` | да | `60` | TTL access token |
| `UNIVERSITY_SIGNING_KEY` | да | `replace-with-signing-secret` | мастер-секрет для генерации подписи вуза |
| `DIPLOMA_LINK_TTL_MINUTES` | да | `30` | TTL временной ссылки на диплом |

### Актуальный `.env.example`

```env
APP_PORT=8080
APP_BASE_URL=http://localhost:8080
RUST_LOG=info,tower_http=info
HR_API_RATE_LIMIT_REQUESTS=60
HR_API_RATE_LIMIT_WINDOW_SECONDS=60
INTEGRATION_API_KEY_BURST_WINDOW_SECONDS=10
INTEGRATION_API_KEY_ATS_ONLY_BURST_LIMIT=30
INTEGRATION_API_KEY_HR_AUTOMATION_ONLY_BURST_LIMIT=20
INTEGRATION_API_KEY_COMBINED_BURST_LIMIT=40
DATABASE_URL=postgres://postgres:postgres@localhost:5432/resume_vizor
DATABASE_MAX_CONNECTIONS=10
REDIS_URL=redis://localhost:6379
REDIS_RATE_LIMIT_PREFIX=resume_vizor:hr_rate_limit
DIPLOMA_HASH_KEY=replace-with-a-long-random-secret
JWT_SECRET=replace-with-another-long-random-secret
ATS_API_KEY_SECRET=replace-with-ats-api-key-secret
JWT_TTL_MINUTES=60
UNIVERSITY_SIGNING_KEY=replace-with-signing-secret
DIPLOMA_LINK_TTL_MINUTES=30
```

## 8. Запуск проекта

### 8.1 Самый быстрый запуск через Docker Compose

В корне проекта уже есть:

- [Dockerfile](/D:/Programming/Resume-visitor/Dockerfile)
- [docker-compose.yml](/D:/Programming/Resume-visitor/docker-compose.yml)
- [.dockerignore](/D:/Programming/Resume-visitor/.dockerignore)

Запуск:

```powershell
docker compose up --build
```

После старта будут подняты:

- `postgres` на `localhost:5432`
- `redis` на `localhost:6379`
- backend на `http://localhost:8080`
- `prometheus` на `http://localhost:9090`
- `grafana` на `http://localhost:3000`

Grafana credentials по умолчанию:

- login: `admin`
- password: `admin`

Остановка:

```powershell
docker compose down
```

Остановка с удалением volume базы:

```powershell
docker compose down -v
```

### 8.2 Что делает Docker Compose

- поднимает PostgreSQL 16
- поднимает Redis 7 для distributed rate limiting
- ждет healthcheck базы
- ждет healthcheck Redis
- собирает backend через multi-stage Docker build
- запускает backend с переменными окружения для подключения к контейнерной БД
- включает Redis-backed limiter для integration API keys
- проверяет readiness backend через `/health/ready`
- поднимает Prometheus с готовым scrape config
- поднимает Grafana с заранее настроенным datasource и dashboard

### 8.3 Настройка secrets для Docker

Перед production-использованием обязательно поменяйте значения:

- `DIPLOMA_HASH_KEY`
- `JWT_SECRET`
- `ATS_API_KEY_SECRET`
- `UNIVERSITY_SIGNING_KEY`

Сейчас в `docker-compose.yml` стоят демонстрационные значения.

### 8.4 Поднять PostgreSQL вручную

Пример через Docker:

```powershell
docker run --name resume-vizor-pg `
  -e POSTGRES_USER=postgres `
  -e POSTGRES_PASSWORD=postgres `
-e POSTGRES_DB=resume_vizor `
  -p 5432:5432 `
  -d postgres:16
```

### 8.5 Создать `.env`

```powershell
Copy-Item .env.example .env
```

При необходимости измените:

- `DATABASE_URL`
- секреты
- `APP_BASE_URL`

### 8.6 Запуск приложения

```powershell
cargo run
```

При старте приложение:

- открывает пул Postgres
- автоматически запускает миграции
- поднимает HTTP сервер

### 8.7 Сборка

Debug:

```powershell
cargo build
```

Release:

```powershell
cargo build --release
```

## 9. Тесты

Тесты используют `in-memory` repository, а не PostgreSQL.

Запуск:

```powershell
cargo test
```

Покрыто:

- регистрация студента
- логин
- авто-привязка диплома студенту
- защита share-link ownership
- разрешение публичной ссылки
- revoke / restore
- HR search
- rate limiter

## 10. Общие правила API

### 10.1 Формат

- все JSON endpoints принимают и возвращают `application/json`
- импорт реестра принимает `multipart/form-data`

### 10.2 Авторизация

Общий формат:

```http
Authorization: Bearer <token>
```

Для role-aware endpoint дополнительно:

```http
role: student
```

### 10.3 Ошибки

Единый формат ошибок:

```json
{
  "error": "validation_error",
  "message": "validation failed: student_number is required"
}
```

Типы ошибок:

- `validation_error`
- `not_found`
- `conflict`
- `unauthorized`
- `forbidden`
- `rate_limited`
- `internal_error`

### 10.4 Статусы диплома

- `active`
- `revoked`

### 10.5 Health и metrics endpoints

Эти endpoint доступны без авторизации:

- `GET /health`
- `GET /health/live`
- `GET /health/ready`
- `GET /metrics`

## 11. Полное описание API

Базовый URL:

```text
http://localhost:8080
```

### 11.1 Health

#### `GET /health`

Совместимый alias для базовой проверки сервиса.

Пример ответа:

```json
{
  "status": "ok",
  "service": "resume-vizor-backend"
}
```

#### `GET /health/live`

Liveness probe.

Назначение:

- показывает, что процесс жив и HTTP слой отвечает

Пример ответа:

```json
{
  "status": "ok",
  "service": "resume-vizor-backend"
}
```

#### `GET /health/ready`

Readiness probe.

Назначение:

- показывает, готово ли приложение принимать трафик
- сейчас readiness проверяет доступность database layer

Успешный ответ:

```json
{
  "status": "ready",
  "service": "resume-vizor-backend",
  "checks": {
    "database": "up"
  }
}
```

Если БД недоступна:

- статус HTTP будет `503 Service Unavailable`

Пример:

```json
{
  "status": "not_ready",
  "service": "resume-vizor-backend",
  "checks": {
    "database": "down"
  }
}
```

#### `GET /metrics`

Prometheus scrape endpoint.

Content-Type:

- `text/plain; version=0.0.4; charset=utf-8`

Пример:

```text
# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",path="/health/live",status="200"} 12
```

### 11.2 Public

#### `GET /api/v1/public/diplomas/access/{token}`

Публичный доступ к диплому по временной ссылке.

Авторизация:

- не требуется

Пример ответа:

```json
{
  "diploma_id": "00000000-0000-0000-0000-000000000001",
  "certificate_id": "00000000-0000-0000-0000-000000000002",
  "university_id": "00000000-0000-0000-0000-000000000003",
  "university_code": "UNI-001",
  "student_number_last4": "1001",
  "diploma_number_last4": "0001",
  "graduation_date": "2026-06-30",
  "record_hash": "....",
  "university_signature": "....",
  "status": "active",
  "revoked_at": null
}
```

### 11.3 Auth

#### `POST /api/v1/auth/register`

Регистрация пользователя.

##### Регистрация студента

```json
{
  "email": "student@example.com",
  "password": "superpass",
  "full_name": "Иван Петров",
  "student_number": "ST-1001",
  "role": "student",
  "university_id": null,
  "university_code": null
}
```

##### Регистрация вуза

```json
{
  "email": "uni@example.com",
  "password": "superpass",
  "full_name": "Университет N",
  "student_number": null,
  "role": "university",
  "university_id": "11111111-1111-1111-1111-111111111111",
  "university_code": "UNI-001"
}
```

##### Регистрация HR

```json
{
  "email": "hr@example.com",
  "password": "superpass",
  "full_name": "HR Manager",
  "student_number": null,
  "role": "hr",
  "university_id": null,
  "university_code": null
}
```

Пример ответа:

```json
{
  "access_token": "<jwt>",
  "token_type": "Bearer",
  "expires_in_seconds": 3600,
  "user": {
    "id": "00000000-0000-0000-0000-000000000001",
    "email": "student@example.com",
    "full_name": "Иван Петров",
    "student_number": "ST-1001",
    "role": "student",
    "university_id": null,
    "university_code": null
  }
}
```

#### `POST /api/v1/auth/login`

```json
{
  "email": "student@example.com",
  "password": "superpass"
}
```

Ответ такой же, как у `register`.

#### `POST /api/v1/auth/change-password`

Авторизация:

- `Authorization: Bearer <jwt>`

Запрос:

```json
{
  "current_password": "superpass",
  "new_password": "new-superpass"
}
```

Ответ:

- `200 OK`
- без body

#### `GET /api/v1/auth/me`

Авторизация:

- `Authorization: Bearer <jwt>`

Ответ:

```json
{
  "id": "00000000-0000-0000-0000-000000000001",
  "email": "student@example.com",
  "full_name": "Иван Петров",
  "student_number": "ST-1001",
  "role": "student",
  "university_id": null,
  "university_code": null
}
```

### 11.4 University API

Все маршруты раздела требуют:

- `Authorization: Bearer <jwt>`
- `role: university`

#### `POST /api/v1/university/diplomas`

Создание одного диплома.

```json
{
  "student_full_name": "Иван Петров",
  "student_number": "ST-1001",
  "student_birth_date": "2003-01-10",
  "diploma_number": "DP-2026-0001",
  "degree": "bachelor",
  "program_name": "computer science",
  "graduation_date": "2026-06-30",
  "honors": false
}
```

Пример ответа:

```json
{
  "diploma_id": "00000000-0000-0000-0000-000000000001",
  "certificate_id": "00000000-0000-0000-0000-000000000002",
  "university_id": "11111111-1111-1111-1111-111111111111",
  "graduation_date": "2026-06-30",
  "diploma_number_last4": "0001",
  "record_hash": "....",
  "university_signature": "....",
  "status": "active",
  "storage_mode": "hashed_only"
}
```

#### `POST /api/v1/university/diplomas/import`

Массовый импорт реестра.

Авторизация:

- `Authorization: Bearer <jwt>`
- `role: university`

Тип:

- `multipart/form-data`

Поле:

- `file`

Пример `curl`:

```bash
curl -X POST "http://localhost:8080/api/v1/university/diplomas/import" \
  -H "Authorization: Bearer <jwt>" \
  -H "role: university" \
  -F "file=@registry.csv"
```

Пример ответа:

```json
{
  "imported_count": 2,
  "failed_count": 1,
  "imported": [
    {
      "row_number": 1,
      "diploma_id": "00000000-0000-0000-0000-000000000001",
      "certificate_id": "00000000-0000-0000-0000-000000000002",
      "record_hash": "....",
      "university_signature": "...."
    }
  ],
  "errors": [
    {
      "row_number": 3,
      "message": "validation failed: student_number is required"
    }
  ]
}
```

#### `POST /api/v1/university/diplomas/{diploma_id}/revoke`

Аннулирование диплома.

Ответ:

```json
{
  "diploma_id": "00000000-0000-0000-0000-000000000001",
  "status": "revoked",
  "revoked_at": "2026-04-03T17:00:00Z"
}
```

#### `POST /api/v1/university/diplomas/{diploma_id}/restore`

Возврат диплома в активный статус.

Ответ:

```json
{
  "diploma_id": "00000000-0000-0000-0000-000000000001",
  "status": "active",
  "revoked_at": null
}
```

### 11.5 Student API

Все маршруты раздела требуют:

- `Authorization: Bearer <jwt>`
- `role: student`

#### `GET /api/v1/student/profile`

Возвращает профиль студента.

Ответ:

```json
{
  "id": "00000000-0000-0000-0000-000000000001",
  "email": "student@example.com",
  "full_name": "Иван Петров",
  "student_number": "ST-1001",
  "role": "student",
  "university_id": null,
  "university_code": null
}
```

#### `POST /api/v1/student/search`

Поиск диплома студентом.

Важно:

- студент вводит `diploma_number` и/или `student_full_name`
- но auto-claim диплома происходит только если совпали:
  - `student.full_name`
  - `student.student_number`
  - данные в дипломе

Пример запроса:

```json
{
  "diploma_number": "DP-2026-0001",
  "student_full_name": "Иван Петров"
}
```

Пример ответа:

```json
{
  "items": [
    {
      "diploma_id": "00000000-0000-0000-0000-000000000001",
      "certificate_id": "00000000-0000-0000-0000-000000000002",
      "university_id": "11111111-1111-1111-1111-111111111111",
      "university_code": "UNI-001",
      "student_number_last4": "1001",
      "diploma_number_last4": "0001",
      "graduation_date": "2026-06-30",
      "program_name_hash": "....",
      "record_hash": "....",
      "university_signature": "....",
      "status": "active",
      "revoked_at": null
    }
  ]
}
```

#### `POST /api/v1/student/diplomas/{diploma_id}/share-link`

Генерирует временную ссылку на диплом.

Важно:

- работает только для диплома, который уже привязан к текущему студенту

Пример ответа:

```json
{
  "diploma_id": "00000000-0000-0000-0000-000000000001",
  "expires_in_seconds": 1800,
  "access_url": "http://localhost:8080/api/v1/public/diplomas/access/<token>"
}
```

### 11.6 HR API

Все маршруты раздела требуют:

- `Authorization: Bearer <jwt>`
- `role: hr`

#### `POST /api/v1/hr/verify`

Базовая проверка диплома по verification lookup.

Запрос:

```json
{
  "student_full_name": "Иван Петров",
  "student_birth_date": "2003-01-10",
  "diploma_number": "DP-2026-0001"
}
```

Ответ:

```json
{
  "found": true,
  "diploma_id": "00000000-0000-0000-0000-000000000001",
  "certificate_id": "00000000-0000-0000-0000-000000000002",
  "status": "active"
}
```

#### `POST /api/v1/hr/registry/search`

Поиск по реестру по:

- `diploma_number`
- `university_code`

Можно передать одно из полей или оба.

Пример:

```json
{
  "diploma_number": "DP-2026-0001",
  "university_code": "UNI-001"
}
```

Ответ:

```json
{
  "items": [
    {
      "diploma_id": "00000000-0000-0000-0000-000000000001",
      "certificate_id": "00000000-0000-0000-0000-000000000002",
      "university_id": "11111111-1111-1111-1111-111111111111",
      "university_code": "UNI-001",
      "student_number_last4": "1001",
      "diploma_number_last4": "0001",
      "graduation_date": "2026-06-30",
      "record_hash": "....",
      "university_signature": "....",
      "status": "active",
      "revoked_at": null
    }
  ]
}
```

#### `POST /api/v1/hr/automation/verify`

Endpoint для автоматической проверки дипломов системой рекрутинга.

Работает так же, как `registry/search`, но вызывается через integration API key.

Требует заголовок:

```http
x-api-key: rv_ats_...
```

Запрос:

```json
{
  "diploma_number": "DP-2026-0001",
  "university_code": "UNI-001"
}
```

Rate limiting:

- лимит считается по конкретному integration API key
- это именно дневная квота, а не rolling window
- счетчик сбрасывается с началом нового UTC-дня
- дополнительно применяется burst limiter, чтобы ключ не мог сделать большой всплеск запросов подряд
- дневная квота сейчас одинаковая для всех ключей: `1000` запросов в день
- лимит зависит от scope ключа:
- `ats_only`
- `hr_automation_only`
- `combined`
- если настроен `REDIS_URL`, лимиты считаются в Redis и работают между несколькими инстансами backend
- если `REDIS_URL` не задан, backend автоматически переключается на `in-memory` fallback

Важно:

- ключ со scope `hr_automation_only` может ходить только в этот endpoint
- ключ со scope `combined` может ходить и сюда, и в `POST /api/v1/ats/verify`
- Redis-backed limiter выбран для этих endpoints специально: он одновременно держит дневную квоту и burst-защиту от слишком частых запросов подряд
- такой сценарий хорошо объясняется жюри: PostgreSQL хранит реестр дипломов, Redis защищает высокочастотный ATS/API traffic и готовит сервис к горизонтальному масштабированию

#### `POST /api/v1/hr/api-keys`

HR выпускает новый integration API key.

Запрос:

```json
{
  "name": "Greenhouse production",
  "scope": "combined"
}
```

Ответ содержит raw `api_key` только один раз при создании.
Также в ответе возвращаются `daily_request_limit`, `burst_request_limit` и `burst_window_seconds`.
Сейчас `daily_request_limit` всегда равен `1000`, а в будущем это место можно расширить до платных планов.

Scope может быть:

- `ats_only`
- `hr_automation_only`
- `combined`

#### `GET /api/v1/hr/api-keys`

Возвращает список integration API keys текущего HR-пользователя.

#### `POST /api/v1/hr/api-keys/{api_key_id}/revoke`

Отзывает integration API key. После этого интеграция больше не сможет ходить в machine-to-machine endpoints.

#### `POST /api/v1/ats/verify`

Machine-to-machine endpoint для ATS.

Требует заголовок:

```http
x-api-key: rv_ats_...
```

Пример запроса:

```json
{
  "diploma_number": "DP-2026-0001",
  "university_code": "UNI-001",
  "candidate_reference": "candidate-42",
  "resume_reference": "resume-42"
}
```

Пример ответа:

```json
{
  "decision": "verified",
  "verified": true,
  "match_count": 1,
  "checked_at": "2026-04-04T12:00:00Z",
  "candidate_reference": "candidate-42",
  "resume_reference": "resume-42",
  "integration_name": "Greenhouse production",
  "risk_flags": [],
  "items": []
}
```

`decision` может быть:

- `verified`
- `manual_review`
- `not_found`

Ключи со scope `ats_only` и `combined` могут вызывать этот endpoint.

## 12. Observability и эксплуатация

### 12.1 Prometheus metrics

Приложение экспортирует Prometheus-совместимые метрики по адресу:

```text
GET /metrics
```

Сейчас собираются:

- `http_requests_total`
  - общее количество HTTP-запросов
  - labels:
    - `method`
    - `path`
    - `status`
- `http_request_duration_seconds`
  - latency запросов в секундах
  - labels:
    - `method`
    - `path`
    - `status`
- `http_requests_in_flight`
  - текущее количество запросов в обработке

Реализация:

- [metrics.rs](/D:/Programming/Resume-visitor/src/infrastructure/metrics.rs)

### 12.2 Health probes

Для инфраструктурного мониторинга и orchestration доступны:

- `GET /health/live`
- `GET /health/ready`

Назначение:

- `live`
  - использовать как liveness probe
- `ready`
  - использовать как readiness probe

### 12.3 Локальный observability stack через Docker Compose

В репозитории уже добавлены:

- [prometheus.yml](/D:/Programming/Resume-visitor/prometheus/prometheus.yml)
- [datasource.yml](/D:/Programming/Resume-visitor/grafana/provisioning/datasources/datasource.yml)
- [dashboards.yml](/D:/Programming/Resume-visitor/grafana/provisioning/dashboards/dashboards.yml)
- [resume-vizor-overview.json](/D:/Programming/Resume-visitor/grafana/dashboards/resume-vizor-overview.json)

После запуска `docker compose up --build` доступны:

- backend: `http://localhost:8080`
- Prometheus UI: `http://localhost:9090`
- Grafana UI: `http://localhost:3000`

Grafana уже автоматически:

- подключена к Prometheus datasource
- загружает dashboard `Resume Vizor Overview`

Что проверить после старта:

1. открыть `http://localhost:8080/metrics`
2. открыть `http://localhost:9090/targets`
3. убедиться, что job `resume-vizor-backend` имеет статус `UP`
4. открыть Grafana и найти dashboard `Resume Vizor Overview`

### 12.4 Docker / Compose рекомендации

Для контейнерного запуска рекомендуется:

- liveness probe на `/health/live`
- readiness probe на `/health/ready`
- Prometheus scraping на `/metrics`

### 12.5 Пример scrape config для Prometheus

```yaml
scrape_configs:
  - job_name: "resume-vizor-backend"
    metrics_path: /metrics
    static_configs:
      - targets:
          - localhost:8080
```

### 12.6 Пример probes для Kubernetes

```yaml
livenessProbe:
  httpGet:
    path: /health/live
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

## 13. Формат массовой загрузки реестра

Поддерживаемые форматы:

- `.csv`
- `.xlsx`

Ожидаемые колонки:

| Индекс | Колонка | Обязательность | Пример |
|---|---|---|---|
| `0` | `fio` | обязательна | `Иван Петров` |
| `1` | `student_number` | обязательна | `ST-1001` |
| `2` | `year` | обязательна | `2026` |
| `3` | `specialnost` | обязательна | `Computer Science` |
| `4` | `diploma_number` | обязательна | `DP-2026-0001` |

### Пример CSV

```csv
fio,student_number,year,specialnost,diploma_number
Иван Петров,ST-1001,2026,Computer Science,DP-2026-0001
Мария Сидорова,ST-1002,2026,Data Science,DP-2026-0002
```

### Что происходит при импорте

Для каждой строки:

1. нормализуются данные
2. вычисляются хеши
3. формируется `record_hash`
4. выполняется цифровая подпись записи вузом
5. запись сохраняется в БД

## 14. Архитектурные потоки

### 14.1 Поток вуза

```mermaid
sequenceDiagram
    participant U as University
    participant API as Axum API
    participant S as DiplomaService
    participant H as Hasher/Signer
    participant DB as PostgreSQL

    U->>API: POST /api/v1/university/diplomas
    API->>S: register_diploma(...)
    S->>H: hash_payload(...)
    S->>H: sign_record_hash(...)
    S->>DB: save diploma
    DB-->>S: saved
    S-->>API: diploma response
    API-->>U: record_hash + signature
```

### 14.2 Поток студента

```mermaid
sequenceDiagram
    participant ST as Student
    participant API as Axum API
    participant S as DiplomaService
    participant DB as PostgreSQL

    ST->>API: POST /api/v1/student/search
    API->>S: search_student_diplomas(...)
    S->>DB: search by hash
    S->>S: validate full_name + student_number
    S->>DB: assign student_account_id if empty
    API-->>ST: owned diplomas
```

### 14.3 Поток HR

```mermaid
sequenceDiagram
    participant HR as HR/ATS
    participant API as Axum API
    participant RL as RateLimiter
    participant S as DiplomaService
    participant DB as PostgreSQL

    HR->>API: POST /api/v1/hr/automation/verify
    API->>RL: allow?
    RL-->>API: yes/no
    API->>S: search_hr_registry(...)
    S->>DB: search by registry hashes
    API-->>HR: diploma search result
```

## 15. База данных

### Таблица `users`

Основные поля:

- `id`
- `email`
- `password_hash`
- `full_name`
- `student_number`
- `role`
- `university_id`
- `university_code`
- `created_at`
- `updated_at`

### Таблица `diplomas`

Основные поля:

- идентификаторы:
  - `id`
  - `university_id`
  - `student_id`
  - `certificate_id`
  - `student_account_id`
- бизнес-поля:
  - `university_code`
  - `student_number_last4`
  - `diploma_number_last4`
  - `record_hash`
  - `university_signature`
  - `signature_algorithm`
  - `status`
  - `revoked_at`
  - `issued_at`
  - `created_at`
- search/hash поля:
  - `university_code_hash`
  - `student_full_name_hash`
  - `student_number_hash`
  - `student_birth_date_hash`
  - `diploma_number_hash`
  - `verification_lookup_hash`
  - `degree_hash`
  - `program_hash`
  - `graduation_date_hash`
  - `honors_hash`
  - `canonical_document_hash`

### Индексы

Добавлены индексы по:

- `student_account_id`
- `student_full_name_hash`
- `student_number_hash`
- `diploma_number_hash`
- `university_code_hash`
- `verification_lookup_hash`

## 16. Что еще не реализовано

На текущий момент сознательно не реализовано:

- QR-коды
- отдельные публичные verification keys для вузов
- distributed rate limiter
- OpenAPI / Swagger
- интеграционные тесты с реальной PostgreSQL
 - отдельные бизнес-метрики уровня домена:
   - количество выданных дипломов
   - количество аннулированных дипломов
   - количество автопривязок студентов

## Полезные команды

Сборка:

```powershell
cargo build
```

Проверка типов/сборки:

```powershell
cargo check
```

Тесты:

```powershell
cargo test
```

Запуск:

```powershell
cargo run
```

## Текущее состояние проекта

Сейчас проект:

- компилируется
- поднимается через PostgreSQL по умолчанию
- автоматически применяет миграции
- имеет тесты на бизнес-логику
- сохраняет `in-memory` repository для тестов
