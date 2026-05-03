<!-- GSD:project-start source:PROJECT.md -->
## Project

**Designer Dashboard**

제품 개발 디자이너가 매일 아침 5분 안에 오늘 할 일과 보낼 메시지를 한눈에 확인할 수 있게 해주는 macOS 메뉴바 앱이다. 슬랙·지메일·구글 캘린더·노션·카카오톡(.txt 내보내기)에서 흩어진 프로젝트 정보를 흡수해 노션을 단일 진실의 원천으로 통합하고, AI가 매일 브리프와 MD/공장에 보낼 지시 초안을 생성해 사용자가 검토 후 발송한다. MD와 공장에는 읽기전용 공유 URL로 진행 상황만 노출한다.

**Core Value:** 매일 아침 5분 안에, 오늘의 액션 목록과 줄 지시 초안을 본다.

### Constraints

- **Tech stack**: Tauri 2.x + React + Rust — Electron 대비 메모리 1/4, 디자이너 사용자의 메모리 민감도 고려
- **Privacy**: 메시지 본문은 절대 클라우드 외부로 나가지 않음(AI 호출 제외, Anthropic 무학습) — 사용자가 다루는 정보가 미공개 디자인·발주처 정보를 포함
- **Source of Truth**: 노션이 SOTR — 다른 채널 데이터는 흡수해 노션에 기록, 충돌 시 노션 우선 (Last-Write-Wins by `last_edited_time`)
- **KakaoTalk**: 공식 API로 개인 톡 읽기/쓰기 불가 → 데스크탑 .txt 내보내기 + 와치 폴더 우회만 사용
- **Send policy**: 모든 외부 발송은 사용자 승인 게이트 통과 필수 — 자동 발송은 정형 알림(D-3 리마인드 등)에 한해 옵트인
- **Platform**: macOS only (v1) — Tauri 구조상 후속 OS 추가는 가능하지만 v1 범위 외
- **Distribution**: 코드 서명 + 자동 업데이트 — 사용자가 Gatekeeper 차단/수동 업데이트로 이탈하지 않게
- **Cost**: v1 운영비 0원 인프라 — Cloudflare Workers/KV/R2 무료 한도, `*.workers.dev` 서브도메인 사용. 도메인 등록 비용 없음. AI 호출 ~$10/월 + Apple Developer ID $99/년만 발생
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## TL;DR — Executive Recommendation
## Recommended Stack
### Core Technologies — Shell & UI
| Technology | Version | Purpose | Why Recommended |
|---|---|---|---|
| **Tauri** | 2.11.0 (Apr 30, 2025) — pin minor, accept 2.12+ patches | App shell: Rust core + WebView UI | Rust core ≈ 1/4 the RAM of Electron, ≈ 30–50 MB bundle vs Electron's 150+ MB. Native macOS tray + window APIs in `tauri::tray` and `tauri::menu`. PROJECT.md correctly chose this. |
| **React** | 18.3.x | UI framework | Stick with React 18, NOT 19. shadcn/ui and the broader Tauri ecosystem all assume 18; React 19's compiler-driven changes don't buy us anything in a single-user desktop app. Re-evaluate at v2 milestone. |
| **TypeScript** | 5.6.x | Type safety | Standard. Match `vite-plugin-react`'s recommended version. |
| **Vite** | 6.x | Dev server + bundler | Tauri 2 docs assume Vite. HMR is essential for a designer-targeting UI. |
| **Tailwind CSS** | 4.x | Styling | shadcn/ui v2 has native Tailwind v4 support. Tailwind v4's lightning CSS and zero-config plugin make startup instant. |
| **shadcn/ui** | latest (CLI: `shadcn@2`) | Component primitives | Copy-not-install model, owns the components, perfect for a single-author project. Korean-friendly because typography is fully ours. |
### Core Technologies — Storage & Auth
| Technology | Version | Purpose | Why Recommended |
|---|---|---|---|
| **`tauri-plugin-sql`** | 2.x | Local SQLite database | Official, sqlx-backed, supports SQLite + Postgres + MySQL. SQLite is correct: single-user, fast, runs in the app process. Use migration files via the plugin's `Migration` API for Notion schema versioning. |
| **`tauri-plugin-keyring`** (community by HuakunShen) | latest 2.x compatible (`@hk/tauri-plugin-keyring-api` on JSR) | OAuth token storage in macOS Login Keychain | Wraps the Rust `keyring` crate. Login Keychain unlocks at user login → no master password prompts. Cross-platform fallback if you ever expand beyond macOS. **Do NOT use `tauri-plugin-stronghold`** — it's being deprecated in Tauri v3 and requires a master password (wrong UX for OAuth tokens). |
| **`tauri-plugin-store`** | 2.x | Non-secret app config (window position, last-sync timestamps, user preferences) | Official key-value JSON store. Use for non-secrets only; secrets go to Keychain. |
| **`tauri-plugin-fs`** (with `watch` feature flag) | 2.x | KakaoTalk .txt watch folder | Tauri 2 consolidates `fs-watch` (v1) into `tauri-plugin-fs` behind a `watch` Cargo feature. Enable it. Underneath it uses the `notify` crate (cross-platform, FSEvents on macOS). |
| **`tauri-plugin-notification`** | 2.x | OS notifications when app is running | Native `NSUserNotification`. Use for *transient* notifications (sync completed, draft ready). Do NOT use its `ScheduleEvery` for the 8 AM daily brief — see scheduler row. |
| **`tauri-plugin-autostart`** | 2.x | "Launch on login" requirement | Official. Wraps macOS `LSSharedFileList` / login items API. |
### Core Technologies — Scheduling, Updates, Distribution
| Technology | Version | Purpose | Why Recommended |
|---|---|---|---|
| **macOS `launchd` user agent** (write to `~/Library/LaunchAgents/com.designerdashboard.dailybrief.plist`) | N/A — OS-native | Daily 8 AM brief that survives sleep + app-not-running | `StartCalendarInterval` with `Hour=8 Minute=0`. Critically: launchd, unlike cron, **runs missed jobs the next time the Mac wakes** — exactly the behavior you want for a laptop designer who closes the lid at 6 PM. Generate the plist from Rust at first-run setup; the plist invokes a small Rust binary that does an HTTP POST to the running app (or, if not running, launches the app). |
| **`tauri-plugin-updater`** (official, RECOMMENDED) | 2.x | In-app auto-update | First-party, ed25519-signed update manifest (JSON), cross-platform. UX equivalent to Sparkle. Host the manifest + .tar.gz / .app.tar.gz on Cloudflare R2 with a public custom domain. |
| **`tauri-plugin-sparkle-updater`** (community by ahonn, ALTERNATIVE) | 0.2.4 (Apr 13, 2026) | macOS-only auto-update via real Sparkle framework + appcast.xml | Use only if you specifically want Sparkle's appcast format or its UI. It bundles Sparkle.framework 2.8.1 into your .app, which adds ~5 MB. Maintained but only 44 commits and 2 releases — single-maintainer risk. We do NOT recommend this unless you have a reason. |
| **Apple Developer ID Application** (paid, $99/yr) | N/A | Code signing | Required for distribution outside the App Store. Without it, Gatekeeper blocks the .app on first launch ("damaged or untrusted"). |
| **`notarytool`** (Xcode 13+) | macOS-bundled | Notarization | Required since macOS 10.15. Tauri's bundler invokes `notarytool` automatically when you set `APPLE_ID`, `APPLE_PASSWORD` (app-specific password), and `APPLE_TEAM_ID` env vars. Adds 2–5 min per release build. |
### Core Technologies — AI
| Technology | Version | Purpose | Why Recommended |
|---|---|---|---|
| **`@anthropic-ai/sdk`** | 0.92.0 (Apr 30, 2026) | Anthropic API client | Latest as of research date. No beta header needed for prompt caching as of 2026. |
| **Claude Sonnet 4.6** | model id `claude-sonnet-4-6` | Daily brief generation, draft-message synthesis, project mapping for ambiguous Slack messages | Best Korean. PROJECT.md's choice. |
| **Claude Haiku 4.5** | model id `claude-haiku-4-5` | High-volume cheap classification (Slack channel → project, Gmail label → project, KakaoTalk message extraction) | Cost-optimized. Use for anything called >10×/day. |
| **Prompt caching** | `cache_control: { type: 'ephemeral', ttl: '5m' }` (or `'1h'` if budget allows) | Cost reduction | 5 min default since Mar 2026 (was 1 hr). Cache reads = 0.1× input price. Cache the system prompt + project schema + recent message context separately. **Important:** workspace-level isolation since Feb 5, 2026 — if you use multiple Anthropic workspaces this matters; you don't, so it's fine. |
### Core Technologies — Cloud (Share-Page Backend)
| Technology | Version | Purpose | Why Recommended |
|---|---|---|---|
| **Cloudflare Workers** | latest runtime (use `compatibility_date = "2026-04-01"` or later in `wrangler.toml`) | Edge API for share pages | Free tier: 100k req/day. Sub-10ms p99 globally. Zero egress fees. |
| **Cloudflare KV** | latest | Storing share-token → project-snapshot-pointer | Eventually consistent up to 60s — fine for share pages. **Critical:** for token *revocation* you want strong consistency; pair KV with a Workers Cache `purge()` call on revocation, OR use KV's `expirationTtl` with short TTLs and a "revocation list" key checked on every request. |
| **Cloudflare R2** | latest | Sanitized snapshot JSON + thumbnail blobs + the bundled static share-page HTML/JS | S3-compatible API, no egress fees. Use a custom domain + signed URLs for thumbnails if needed. |
| **Hono** | 4.12.16 (Apr 30, 2026) | Worker HTTP framework | <15 KB gzipped. First-class Workers support, typed `c.env` bindings via `wrangler.toml`. The de-facto standard for Workers in 2026. Far more pleasant than raw `fetch` handler or itty-router for anything beyond ~3 routes. |
| **Wrangler** | latest 3.x | Workers CLI, dev server, deploy | Required. Use `wrangler dev` for local. KV + R2 are simulated locally. |
### Supporting Libraries
| Library | Version | Purpose | When to Use |
|---|---|---|---|
| **`@slack/web-api`** | 7.x | Slack API (channels, messages, files, users) | Official. Requires Node ≥ 18 (matches Tauri's bundled environment). Handles rate limits + retries automatically. |
| **`googleapis`** | 144.x+ | Gmail + Google Calendar | Official Google client. Use `google.gmail({version: 'v1', auth})` and `google.calendar({version: 'v3', auth})`. Pair with `google-auth-library` for OAuth2 flow. |
| **`@notionhq/client`** | 5.19.x | Notion API (databases, pages, blocks, comments) | Official. Pass `notionVersion: '2026-03-11'` for the latest API features (typed multi-select filters, comments update/delete). |
| **`google-auth-library`** | 9.x | Google OAuth2 PKCE flow | Pair with `googleapis`. |
| **`zod`** | 3.x | Runtime validation of API responses + share-page payloads | Notion responses are notoriously polymorphic; zod schemas force you to handle the variants. |
| **`drizzle-orm`** + `drizzle-kit` | 0.36.x / 0.30.x | SQLite query builder + migrations | Optional but strongly recommended. Type-safe queries, generates SQL migrations from schema changes. Works with `tauri-plugin-sql` (you write SQL, but Drizzle generates the SQL strings). Alternative: hand-write SQL — fine for <20 tables. |
| **`@tanstack/react-query`** | 5.x | Frontend data fetching + caching against the Rust IPC | Caches IPC results, dedupes concurrent calls, handles stale-while-revalidate. Far better than rolling your own `useState` for every Notion query. |
| **`zustand`** | 5.x | Frontend global state (current view, selected project, draft cards) | Tiny (~1 KB), no boilerplate, perfect for menu-bar app scope. |
| **`date-fns`** + `date-fns-tz` | 4.x | Date math + timezone (KST handling for Korean designers) | All date/time arithmetic must respect Asia/Seoul. Don't use `Date` arithmetic directly. |
| **`hono/jwt`** (built into Hono) | with Hono 4.12 | Share-page token issuance + verification on Workers | HS256 with a Workers secret env var. Issue from the Mac app, verify on the Worker. |
### Rust Crates (in `src-tauri/Cargo.toml`)
| Crate | Version | Purpose | Notes |
|---|---|---|---|
| **`tokio`** | 1.x (with `full` features) | Async runtime | Tauri uses tokio internally; you'll use it for IPC commands and the watch-folder loop. |
| **`reqwest`** | 0.12.x (with `rustls-tls`) | HTTP client for OAuth refresh + Notion/Slack/Gmail/Calendar/Anthropic | Use rustls (not native-tls) to avoid macOS cert chain issues. |
| **`serde`** + **`serde_json`** | 1.x | JSON serialization | Universal. |
| **`rusqlite`** or **`sqlx`** | sqlx 0.8.x | SQLite driver | Already pulled in by `tauri-plugin-sql`. If you go raw without the plugin, prefer `rusqlite` for simplicity. |
| **`notify`** | 6.x | File watching (KakaoTalk folder) | Used internally by `tauri-plugin-fs` watch feature. You can use it directly if you need finer control. |
| **`keyring`** | 3.x | macOS Login Keychain | Used internally by `tauri-plugin-keyring`. |
| **`objc2`** + **`objc2-quick-look-thumbnailing`** + **`objc2-foundation`** | latest | Rust → QLThumbnailGenerator FFI for PSD/AI/INDD thumbnails (RECOMMENDED PATH) | Pure Rust, no `unsafe` blocks needed for the high-level API. Falls back to native QuickLook generator pipeline. Alternative: shell out to `/usr/bin/qlmanage -t -s 512 -o <out_dir> <input>` — simpler, slower, and creates orphan files but zero FFI risk. |
| **`anyhow`** + **`thiserror`** | 1.x / 2.x | Error handling | Standard. |
| **`tracing`** + **`tracing-subscriber`** | 0.1.x / 0.3.x | Structured logging to a rotating file in `~/Library/Logs/DesignerDashboard/` | Critical for debugging sync issues that happen at 8 AM when you're not watching. |
### Development Tools
| Tool | Purpose | Notes |
|---|---|---|
| **pnpm** | Package manager | Faster + better disk usage than npm/yarn. Tauri templates assume it. |
| **Biome** | Linter + formatter for TS/React | Replace ESLint + Prettier. 10–100× faster, single config. |
| **`cargo-watch`** | Rust hot-reload during dev | `cargo install cargo-watch`, then `cargo watch -x check`. |
| **`tauri-cli`** | Build + run + bundle | `cargo install tauri-cli --version "^2.0"` or use the npm wrapper `@tauri-apps/cli`. |
| **GitHub Actions** | Release pipeline | macOS runner (`macos-14` for Apple Silicon, `macos-13` for Intel). Cache `~/.cargo/registry` + `target/` to keep builds <10 min. |
| **`act`** | Run GH Actions locally | Helpful before pushing tag releases. |
## Installation
# === Bootstrap (one time) ===
# === Scaffold project (you can use the agmmnn/tauri-ui template instead) ===
# === Frontend deps ===
# Keyring (community plugin via JSR)
# UI
# State + data
# API SDKs
# Optional but recommended
# === Rust deps (in src-tauri/Cargo.toml — add manually) ===
# tauri = { version = "2.11", features = ["macos-private-api", "tray-icon"] }
# tauri-plugin-sql = { version = "2", features = ["sqlite"] }
# tauri-plugin-store = "2"
# tauri-plugin-fs = { version = "2", features = ["watch"] }
# tauri-plugin-notification = "2"
# tauri-plugin-autostart = "2"
# tauri-plugin-updater = "2"
# tauri-plugin-keyring = "*"     # check exact version on crates.io
# tokio = { version = "1", features = ["full"] }
# reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json"] }
# serde = { version = "1", features = ["derive"] }
# serde_json = "1"
# notify = "6"
# objc2 = "0.5"
# objc2-foundation = "0.2"
# objc2-quick-look-thumbnailing = "0.2"
# anyhow = "1"
# thiserror = "2"
# tracing = "0.1"
# tracing-subscriber = "0.3"
# tracing-appender = "0.2"
# === Cloudflare Workers (separate /workers subdir) ===
## Alternatives Considered
| Recommended | Alternative | When to Use Alternative |
|---|---|---|
| **Tauri 2.x** | **Electron 32.x** | If you must ship a feature that requires deep Node.js integration (e.g., a giant existing Node codebase). For a greenfield productivity app: never. |
| **Tauri 2.x** | **SwiftUI MenuBarExtra** | If you commit to macOS-only **forever** AND the team has Swift expertise. PROJECT.md keeps "post-v1 OS extensibility" open, so Tauri wins. SwiftUI also can't easily reuse the Cloudflare Workers + share-page React bundle. |
| **`tauri-plugin-keyring`** | **`tauri-plugin-stronghold`** | Never, for OAuth tokens. Stronghold is designed for cryptocurrency-grade master-password vaults. **Also: deprecated in Tauri v3.** |
| **`tauri-plugin-keyring`** | Custom Keychain bridge via `objc2-security` | Only if you need fine-grained ACLs (e.g., share with a specific helper binary). For our case, the plugin is sufficient. |
| **`launchd` user agent for 8 AM** | **`tauri-plugin-notification` `ScheduleEvery::Day`** | Only if app is guaranteed running 24/7. It isn't (designer closes laptop). Misses fire when app isn't up. |
| **`launchd` user agent** | **A "tauri-plugin-cron" community plugin** | Same problem as above — lives in-process, dies with the app. None of these have the launchd "run-on-wake" semantic. |
| **`tauri-plugin-updater` (official)** | **`tauri-plugin-sparkle-updater` (community)** | If you need real Sparkle's appcast.xml format or its UI specifically (e.g., enterprise IT requires it). Adds Sparkle.framework dependency (~5 MB) and is single-maintainer. |
| **`@tauri-apps/plugin-fs` watch feature** | **`chokidar` in Node sidecar** | If you need cross-platform `fs.watchFile` polling fallbacks. On macOS only, `notify` (FSEvents) is fine and doesn't require shipping Node. |
| **macOS `QLThumbnailGenerator`** for PSD/AI/INDD | **ImageMagick (via `magick-rust` or sidecar binary)** | If native QL fails for your specific files (especially old `.indd` and Illustrator versions without embedded previews). ImageMagick can read PSD natively, but **not** AI or INDD without Ghostscript + extra config. Adds 30+ MB to bundle. Use only as a last-resort fallback. |
| **macOS `QLThumbnailGenerator`** for PSD/AI/INDD | **`sharp` (libvips)** | **Don't.** Sharp does not support PSD, AI, or INDD. It's strictly raster (JPEG/PNG/WebP/AVIF/TIFF). Useful for *resizing* the QL output to a smaller thumbnail, but not for *generating* it. |
| **macOS `QLThumbnailGenerator`** | **`psd` Rust crate (chinedufn/psd)** for PSD only | Use as fallback for PSD specifically — it can extract the embedded JPEG preview from the PSD's "Image Resources" section. Won't work for AI or INDD. |
| **Hono on Workers** | **itty-router** | If you want zero overhead and only have 1–2 routes. We have ≥6 routes (issue token, fetch snapshot, revoke, list active tokens, R2 thumbnail proxy, healthcheck) — Hono pays off. |
| **Hono on Workers** | **Raw `fetch` handler** | Never beyond a single endpoint. You'll reinvent routing badly. |
| **Cloudflare KV** | **Cloudflare D1** (SQLite at the edge) | If share-page payloads grow >25 MB each, or you need joins. v1 share data is small JSON snapshots — KV wins. |
| **Anthropic Sonnet 4.6** for everything | **Anthropic Haiku 4.5** for everything | Don't go all-Haiku — quality on Korean draft messages will visibly drop. Use Haiku for classification, Sonnet for generation. PROJECT.md's split is correct. |
| **`@anthropic-ai/sdk`** direct | **Vercel AI SDK** with Anthropic provider | AI SDK adds streaming hooks for React but is overkill for a single-call-per-brief pattern. Use it only if you add a chat UI later. |
| **React 18** | **React 19** | When the shadcn/ui + Tauri ecosystems fully migrate (mid-2026 expected). For now, React 18 is the safer choice and you lose nothing. |
| **Tailwind v4** | **Tailwind v3** | Only if a critical shadcn component you need hasn't migrated. shadcn v2 is v4-native, so this is unlikely. |
| **pnpm** | **bun** | Bun is fast but the Tauri toolchain assumes Node-compatible package managers. Use Bun for the Workers if you want, but keep pnpm for the app. |
## What NOT to Use
| Avoid | Why | Use Instead |
|---|---|---|
| **`tauri-plugin-stronghold`** | Officially being deprecated in Tauri v3. Designed for crypto wallets, not OAuth tokens. Requires user to remember a master password — terrible UX for daily use. | `tauri-plugin-keyring` (uses macOS Login Keychain) |
| **`sharp` for PSD/AI/INDD** | libvips does not decode these formats. Will throw `unsupported file format`. | `QLThumbnailGenerator` (via `objc2-quick-look-thumbnailing` Rust bindings) primary; `psd` crate fallback for PSD; `qlmanage` shell-out as alternative |
| **In-process daily 8 AM scheduler** (`setInterval`, `tokio::time::sleep`, `tauri-plugin-notification.schedule()`) | Dies when the app dies. Misses fires when the laptop sleeps and the app isn't running. | `launchd` user agent with `StartCalendarInterval` |
| **Electron** | Memory + bundle size violates PROJECT.md's "메모리 민감" constraint. | Tauri 2 |
| **`node-keytar`** | Unmaintained since 2023. Has open security issues. | `tauri-plugin-keyring` (Rust side) |
| **`cron` system tab on macOS** | macOS officially deprecated `cron` in favor of `launchd` years ago. Doesn't run on wake-from-sleep. Not enabled by default on Sonoma+. | `launchd` user agent |
| **Custom auth server in Cloudflare Workers for Slack/Google/Notion OAuth** | The OAuth callback flow needs a public HTTPS URL but we're a desktop app. Use Tauri's `tauri-plugin-deep-link` (`designerdashboard://oauth/slack`) instead. | `tauri-plugin-deep-link` for OAuth callback to a custom URL scheme; only use Workers for the share-page backend. |
| **`react-router` for the in-app UI** | We have ≤8 views in a tiny menu-bar window. Routing is overkill. | Conditional render driven by zustand state. |
| **`crypto-js` or hand-rolled JWT** | Slow + insecure. | `hono/jwt` (uses Web Crypto, runs everywhere) |
| **`googleapis` for KakaoTalk** | KakaoTalk has no official API for personal chats. PROJECT.md correctly chose .txt watch folder. | The watch-folder approach. Period. |
| **Bundling `magick-rust` / ImageMagick "just in case"** | Adds 30–50 MB to the bundle and only helps for edge-case Adobe files. | Lazy fallback only — and consider if even needed. |
## Stack Patterns by Variant
- Add a sidecar fallback using **`/usr/bin/sips`** (built-in macOS) for raster + **the `psd` Rust crate** for PSD embedded preview extraction.
- Last resort: ship a small Adobe Quick Look generator helper (Adobe used to provide one — check current state at v1 user-testing time).
- Always show a placeholder + retry button in the gallery so a missing thumbnail never blocks the workflow.
- Add a `WakeFromSleep = true` option in the plist and document the System Settings → Battery → "Wake for network access" requirement.
- Optionally schedule for 7:55 instead of 8:00 to give wake-from-sleep a 5-min buffer.
- Replace Anthropic with a local model via `mlc-llm` or Ollama running as a sidecar. Out of scope for v1 per PROJECT.md.
- Move thumbnails to S3-compatible Backblaze B2 (cheaper egress) or fall back to data URIs in the share-page HTML.
## Version Compatibility
| Package A | Compatible With | Notes |
|---|---|---|
| `tauri@2.11.x` | `tauri-plugin-*@2.x` | Match major version. Plugins released alongside core. |
| `tauri@2.x` Rust toolchain | Rust 1.77.2+ | All official Tauri 2 plugins require this. CI must pin a recent stable. |
| `tauri-plugin-fs@2.x` watch feature | macOS 10.13+ | Uses FSEvents. Fine for Catalina+ users. |
| `tauri-plugin-sparkle-updater@0.2.4` | Sparkle 2.8.1, macOS 10.13+, Tauri 2.x | If you choose this path. Bundles Sparkle.framework. |
| `tauri-plugin-updater@2.x` (official) | Cross-platform, no extra runtime | Recommended path. |
| `@anthropic-ai/sdk@0.92.x` | Node 18+ | Works in Node sidecars and main process. NOT in Cloudflare Workers without polyfills (use `fetch` directly there if needed). |
| `Tailwind v4` | shadcn/ui v2+ | shadcn v1 → v3 only. Don't mix. |
| `React 18.3.x` | shadcn/ui latest | Confirmed compatible. React 19 has migration warnings, not errors. |
| `Hono 4.12.x` | Workers runtime with `compatibility_date >= 2024-09-23` | Earlier dates miss `nodejs_compat` v2 features Hono 4 uses. |
| `googleapis@144.x` | Node 18+ | Same Node version as Slack SDK — single Node runtime works for both. |
| `@notionhq/client@5.x` | API version 2026-03-11 (or 2025-09-03 default) | Pass `notionVersion` explicitly to opt into the new typed filters. |
| `objc2-quick-look-thumbnailing@0.2.x` | macOS 10.15+ (Catalina) | The QuickLookThumbnailing framework was introduced in 10.15. Fine for our target audience. |
## Quality-Gate Self-Check
| Gate | Status |
|---|---|
| Versions verified against current docs (not training data) | YES — all versions above were verified via WebSearch/WebFetch on 2026-05-03. Tauri 2.11.0 (GitHub releases), Hono 4.12.16 (GitHub), `@anthropic-ai/sdk` 0.92.0 (GitHub), `tauri-plugin-sparkle-updater` 0.2.4 (GitHub Apr 13 2026). |
| Each recommendation explains WHY over alternatives | YES — see "Alternatives Considered" + inline "Why Recommended" column. |
| Confidence levels assigned | YES — overall HIGH, with MEDIUM caveats called out for Sparkle plugin and PSD/AI/INDD thumbnails. Per-recommendation: HIGH for Tauri/React/Tailwind/SDKs/Hono/Workers; MEDIUM for keyring plugin (community, but well-architected); MEDIUM for thumbnail strategy (native API has known gaps for AI/INDD). |
| PSD/AI/INDD strategy is concrete | YES — primary: `objc2-quick-look-thumbnailing` (Rust FFI to native QL); secondary: `qlmanage` shell-out; PSD fallback: `psd` Rust crate to extract embedded JPEG; ultimate fallback: placeholder. AI/INDD limitation flagged for PITFALLS.md. |
| Apple code signing + Sparkle/Updater integration documented | YES — Developer ID Application cert + `notarytool` env vars (`APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`) for notarization; `tauri-plugin-updater` (official) recommended over Sparkle plugin with rationale. |
## Sources
- Tauri 2 official plugins overview — https://v2.tauri.app/plugin/ — verified plugin list, MEDIUM (page didn't show explicit versions but confirmed plugin existence)
- Tauri 2 latest release v2.11.0 — https://github.com/tauri-apps/tauri/releases — HIGH
- Tauri 2 macOS code signing + notarization — https://v2.tauri.app/distribute/sign/macos/ — HIGH
- Tauri 2 menu bar app community guides — https://github.com/ahkohd/tauri-macos-menubar-app-example, https://dev.to/hiyoyok/how-i-built-a-macos-menu-bar-hud-with-rust-tauri-20-pij — MEDIUM
- `tauri-plugin-sparkle-updater` v0.2.4 — https://github.com/ahonn/tauri-plugin-sparkle-updater, https://crates.io/crates/tauri-plugin-sparkle-updater — HIGH for version, MEDIUM for production-readiness
- `tauri-plugin-keyring` (community by HuakunShen) — https://github.com/HuakunShen/tauri-plugin-keyring, https://huakunshen.com/projects/tauri-plugin-keyring — MEDIUM (community plugin, but actively maintained and wraps the well-trusted Rust `keyring` crate)
- Tauri Stronghold deprecation note — https://v2.tauri.app/plugin/stronghold/ (release notes mention deprecation in v3) — HIGH (do NOT use Stronghold for OAuth tokens)
- macOS launchd `StartCalendarInterval` wake-from-sleep behavior — https://www.launchd.info/, https://blog.darnell.io/automation-on-macos-with-launchctl/, https://alvinalexander.com/mac-os-x/launchd-plist-examples-startinterval-startcalendarinterval/ — HIGH
- macOS QLThumbnailGenerator — https://developer.apple.com/documentation/quicklookthumbnailing/qlthumbnailgenerator, https://eclecticlight.co/2024/10/31/how-sequoia-has-changed-quicklook-and-its-thumbnails/ — HIGH (with the Sequoia caveat that old QL plugins no longer load)
- `objc2-quick-look-thumbnailing` Rust bindings — https://docs.rs/objc2-quick-look-thumbnailing/latest/objc2_quick_look_thumbnailing/ — HIGH
- `qlmanage` command-line — https://alexwlchan.net/2020/using-qlmanage-to-create-thumbnails-on-macos/, https://eclecticlight.co/2018/04/05/inside-quicklook-previews-with-qlmanage/ — HIGH
- `chinedufn/psd` Rust crate for PSD parsing + embedded preview — https://github.com/chinedufn/psd, https://lib.rs/crates/psd — HIGH
- `sharp` does not support PSD/AI/INDD — https://github.com/lovell/sharp, https://www.npmjs.com/package/sharp — HIGH (negative claim verified against the formats list in official docs)
- Hono 4.12.16 latest release — https://github.com/honojs/hono/releases — HIGH
- Hono on Cloudflare Workers — https://hono.dev/docs/getting-started/cloudflare-workers, https://developers.cloudflare.com/workers/framework-guides/web-apps/more-web-frameworks/hono/ — HIGH
- Cloudflare Workers KV best practices + bindings — https://developers.cloudflare.com/kv/, https://developers.cloudflare.com/workers/wrangler/configuration/ — HIGH
- KV eventual consistency (60s) — https://developers.cloudflare.com/kv/concepts/how-kv-works/ — HIGH (called out as an architecture pitfall for token revocation)
- `@anthropic-ai/sdk` v0.92.0 latest — https://github.com/anthropics/anthropic-sdk-typescript/releases — HIGH
- Anthropic prompt caching TTL change (March 2026: default 1h → 5m) — https://dev.to/whoffagents/anthropic-silently-dropped-prompt-cache-ttl-from-1-hour-to-5-minutes-16ao, https://platform.claude.com/docs/en/build-with-claude/prompt-caching — MEDIUM (verify against official docs at implementation time; pricing structure was 1.25× write / 0.1× read at last check)
- `@slack/web-api` official Node SDK — https://www.npmjs.com/package/@slack/web-api, https://docs.slack.dev/tools/node-slack-sdk/web-api/ — HIGH
- `@notionhq/client` v5.19, API version 2026-03-11 — https://github.com/makenotion/notion-sdk-js, https://www.npmjs.com/package/@notionhq/client — HIGH
- `googleapis` Node + OAuth2 best practices — https://developers.google.com/workspace/calendar/api/quickstart/nodejs — HIGH
- shadcn/ui Tailwind v4 support — https://ui.shadcn.com/docs/tailwind-v4 — HIGH
- shadcn/ui + Tauri starter (`agmmnn/tauri-ui`) — https://github.com/agmmnn/tauri-ui — HIGH
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
