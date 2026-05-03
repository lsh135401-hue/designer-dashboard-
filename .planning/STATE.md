# State: Designer Dashboard

**Last updated:** 2026-05-03
**Status:** Roadmap created — ready to plan Phase 0

## Project Reference

**Core value**: 매일 아침 5분 안에, 오늘의 액션 목록과 줄 지시 초안을 본다.

**Current focus**: Foundation primitives (tray + autostart + Keychain + SQLite WAL + OAuth single-flight + AI budget primitive + Notion rate limiter + Hardened Runtime + wake observer) before any channel or AI integration ships.

**Mode**: yolo · standard granularity · parallelization enabled · balanced model profile

**Constraints active**:
- Tauri 2.x (Rust + React) — memory-sensitive users
- Notion is SOTR; LWW by `last_edited_time`
- Message bodies stay local (SQLite + Keychain); never to Cloudflare KV
- All outbound sends require user approval gate
- macOS only for v1
- v1 infra cost target ≈ $0/month + ~$10/month AI

## Current Position

| Field | Value |
|-------|-------|
| Current phase | 0 — Foundation Shell |
| Current plan | (none — phase not yet planned) |
| Current node | — |
| Status | Partial — tray/popover shell built; core Rust primitives not yet implemented |
| Phase progress | 0/9 phases complete (Phase 0 partial) |
| Plans complete in current phase | 0/0 (plans not yet formally derived via `/gsd-plan-phase 0`) |
| Last completed phase | — |
| Next milestone gate | Phase 0 verification (see ROADMAP.md success criteria) |

```
Roadmap progress
[░░░░░░░░░░░░░░░░░░░░] 0/9 phases (0%)

Current phase plans
(plans not yet derived)
```

## Performance Metrics

| Metric | Current | Target | Notes |
|--------|---------|--------|-------|
| v1 requirements mapped | 100/100 | 100 | All REQ-IDs assigned to phases |
| Phases planned | 0/9 | 9 | Run `/gsd-plan-phase 0` next |
| Code shipped | ~350 LOC Rust + ~1200 LOC HTML/JS | — | Tray + popover shell only; no Phase 0 Rust primitives yet |
| Notarization smoke test | not yet | passing in CI | P0 verification gate |
| AI daily spend | $0 | <$1/day default ceiling | COST-01 enforced from P0 |
| Prompt cache hit rate | — | ≥70% | COST-07; first measurable in P3 |

## Accumulated Context

### Decisions Logged

| ID | Decision | Source | Rationale |
|----|----------|--------|-----------|
| D-01 | 9-phase structure (P0–P8) | research/SUMMARY.md | Derived from requirement coverage + dependency ordering |
| D-02 | Privacy boundary (`SharedView`) lands in P2 before any channel | research/ARCHITECTURE.md Pattern 5 | Compile-time enforcement vs code-review burden |
| D-03 | AI budget primitive in P0, first exercised in P3 | research/PITFALLS.md #12 | Avoids retrofitting cost guardrails after AI calls already exist |
| D-04 | OAuth single-flight mutex in P0 | research/PITFALLS.md #6 | Adding mutex later requires touching every call site |
| D-05 | Scheduler: launchd `RunAtLoad=true` + in-app tokio + wake observer (NOT `StartCalendarInterval`) | research/SUMMARY.md Conflict 1 | Avoids duplicate-process anti-pattern + needs in-memory aggregated state |
| D-06 | Auto-update via `tauri-plugin-updater` (official, ed25519) over `tauri-plugin-sparkle-updater` | research/SUMMARY.md Conflict 2 | First-party, smaller bundle, lower single-maintainer risk; treat PROJECT.md "Sparkle" as shorthand |
| D-07 | Notion webhooks deferred to P8 as additive over polling | research/SUMMARY.md Conflict 3 | Polling alone meets NOTION-04 5-min sync; webhooks are defense in depth |
| D-08 | P4 (Gmail+GCal) and P5 (KakaoTalk+Adobe) parallelizable after P3 | config.json `parallelization=true` | No upstream dependency between them once AI patterns are set |
| D-09 | Granularity = standard → 9 phases | config.json | Within 5–8 normal range; +1 because P0 is a primitives-only setup phase that doesn't ship user-visible features |

### Open Todos

(none — roadmap is the immediate next artifact; planning starts at P0)

### Active Blockers

(none)

### Open Questions Surfaced by Research (informational; defaults assumed unless user objects)

| Question | Default Assumed |
|----------|-----------------|
| Auto-update mechanism (Conflict 2) | `tauri-plugin-updater` (official) — D-06 above |
| Daily brief default time | 8:00 (user-configurable per BRIEF-01) |
| KakaoTalk watch folder default | NSOpenPanel-picked, suggested `~/Documents/KakaoTalk/` |
| Confidentiality tier default for new projects | `internal` (one-click promotion to `shared`) |
| AI daily budget default | $1.00/day (user-raisable) |
| Share URL default expiry | 7 days (one-click extend) |
| Multi-track progress UI | v1.x — v1 ships single % per project |
| JANDI / Swit support | v2 |
| Onboarding skip-step UX | All steps deferrable with "나중에 연결" badge (ONBOARD-02) |

## Session Continuity

**Session 1** (2026-05-03, 30 commits): Initial roadmap + research synthesis (PROJECT.md, REQUIREMENTS.md, ROADMAP.md, research/). Built v0 web mockup (5-section SPA + 3 layout variants + Cmd+K search + color hierarchy + +추가 routing to external services). Pushed to GitHub Pages (https://lsh135401-hue.github.io/designer-dashboard-/). Scaffolded Tauri 2 menu-bar app at `app/`: macOS Accessory mode, tray icon, 320×720 transparent popover, hide-on-blur, position-under-tray, settings UI mock (Notion/Google/Slack/KakaoTalk), KakaoTalk .txt drag-drop parser with AI summary mock. App installed to `/Applications/Designer Dashboard.app`, DMG built. v0.1.0 tag pushed.

**What Phase 0 still needs**: Keychain (`tauri-plugin-keyring`), SQLite WAL (`tauri-plugin-sql`), OAuth single-flight mutex, AI budget primitive, Notion rate limiter, wake observer, Hardened Runtime entitlements + CI notarization smoke test.

**Next session entry point**: `/gsd-plan-phase 0` — derive formal plans for the Phase 0 Rust primitives above. Tray shell is done; Rust infrastructure layer is the outstanding work. Then `/gsd-plan-phase 1` for Notion OAuth integration (the user's stated goal: replace mockup data with real Notion DB).

**Files to load on resume**:
- `.planning/PROJECT.md`
- `.planning/REQUIREMENTS.md`
- `.planning/ROADMAP.md`
- `.planning/research/SUMMARY.md` (Phase 0 section)
- `.planning/research/STACK.md` (plugin set + entitlements)
- `.planning/research/PITFALLS.md` (Pitfalls 6, 14 — directly relevant to P0)

---
*State initialized: 2026-05-03*
