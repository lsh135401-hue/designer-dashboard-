# Requirements: Designer Dashboard

**Defined:** 2026-05-03
**Core Value:** 매일 아침 5분 안에, 오늘의 액션 목록과 줄 지시 초안을 본다

## v1 Requirements

Requirements for initial release. Each maps to a roadmap phase (filled by gsd-roadmapper).

### Foundation (FOUND)

- [ ] **FOUND-01**: macOS 메뉴바 트레이 아이콘이 항상 표시되며 클릭 시 메인 윈도우가 200ms 안에 열린다
- [ ] **FOUND-02**: macOS 로그인 시 앱이 자동 시작되며 사용자가 설정에서 끌 수 있다
- [ ] **FOUND-03**: OAuth 토큰이 macOS Keychain에 저장되고 앱 재시작 후에도 사용 가능하다
- [ ] **FOUND-04**: 글로벌 단축키(기본 ⌥+Space, 사용자 변경 가능)로 어디서든 메인 윈도우를 호출할 수 있다
- [ ] **FOUND-05**: 동시에 여러 OAuth 토큰 갱신이 발생해도 single-flight 보호로 토큰이 손상되지 않는다
- [ ] **FOUND-06**: macOS sleep/wake 후에도 모든 채널 동기화 커서가 유지되고 wake 시 자동 재동기화한다

### Notion Sync (NOTION)

- [ ] **NOTION-01**: 사용자가 노션 통합 토큰을 입력하면 마법사가 프로젝트 DB를 자동 발견한다 (data_source_id 우선)
- [ ] **NOTION-02**: 노션 DB에 필수 속성(Name/Status/Due Date)이 없으면 마법사가 자동 추가한다
- [ ] **NOTION-03**: 12단계 패션 마일스톤 템플릿(기획→1차시안→…→출고)이 첫 실행 시 자동 생성되고 사용자가 단계 추가/삭제/이름변경할 수 있다
- [ ] **NOTION-04**: 노션에서 프로젝트 생성/수정 시 5분 안에 앱에 반영된다
- [ ] **NOTION-05**: 앱에서 마일스톤·태스크 체크 시 즉시 노션에 반영된다
- [ ] **NOTION-06**: 동시 편집 충돌 시 노션이 우선(Last-Write-Wins)되고 손실된 로컬 변경은 Communication Log에 audit으로 기록된다
- [ ] **NOTION-07**: 4-state notion_state(active/archived/trashed/gone) 추적으로 아카이브된 프로젝트가 ghost로 남지 않는다
- [ ] **NOTION-08**: 사용자가 노션에서 속성 이름을 바꿔도 schema-by-id 매핑이 깨지지 않고 "이름 변경 감지" UI를 띄운다
- [ ] **NOTION-09**: API 호출은 2.5 req/s 토큰 버킷으로 제한되어 429 storm이 발생하지 않는다

### Google Calendar (GCAL)

- [ ] **GCAL-01**: 구글 OAuth로 캘린더에 접근하고 향후 30일 이벤트를 가져온다
- [ ] **GCAL-02**: 캘린더 이벤트 제목·설명에 프로젝트명이 포함되면 자동으로 해당 프로젝트에 매핑된다
- [ ] **GCAL-03**: 앱에서 마일스톤 due_date를 변경하면 90초 안에 캘린더 이벤트가 생성/갱신된다
- [ ] **GCAL-04**: syncToken 기반 incremental sync로 변경된 이벤트만 가져온다

### Gmail (GMAIL)

- [ ] **GMAIL-01**: 구글 OAuth(캘린더와 단일 토큰)로 Gmail에 접근한다
- [ ] **GMAIL-02**: 라벨링된 메일이 100% 자동으로 해당 프로젝트에 분류된다
- [ ] **GMAIL-03**: 라벨이 없으면 발신자/제목 매칭으로 분류 시도, 실패 시 "어디에 속해요?" 큐에 들어간다
- [ ] **GMAIL-04**: thread-level 분류로 한 스레드가 두 프로젝트로 갈라지지 않는다
- [ ] **GMAIL-05**: 메일 본문은 로컬에만 저장되고 클라우드로 절대 전송되지 않는다
- [ ] **GMAIL-06**: historyId 기반 incremental sync로 인박스 전체 재스캔 없이 신규 메일만 처리한다

### Slack (SLACK)

- [ ] **SLACK-01**: Slack OAuth(Bot+User token)로 워크스페이스에 접근한다
- [ ] **SLACK-02**: Socket Mode WebSocket으로 실시간 수신, 오프라인 시 Cloudflare Worker bridge에서 누락 이벤트를 catch-up한다
- [ ] **SLACK-03**: 같은 메시지 이벤트가 두 경로로 도착해도 (channel_id, event_ts) unique로 dedup된다
- [ ] **SLACK-04**: message / message_changed / message_deleted / bot_message subtype을 모두 처리한다
- [ ] **SLACK-05**: Slack 핸들러는 100ms 안에 ACK를 반환한다 (3초 timeout 안전)
- [ ] **SLACK-06**: 메시지 → 프로젝트 매핑 정확도 90%↑ (채널명/본문/AI 분류 우선순위)
- [ ] **SLACK-07**: AI 분류 신뢰도 0.7 미만은 "어디에 속해요?" 수동 검토 큐로 보낸다

### Adobe Asset Recognition (ADOBE)

- [ ] **ADOBE-01**: Slack에 업로드된 .psd/.ai/.indd/.sketch/.fig/.png/.jpg/.pdf가 자동으로 프로젝트 갤러리에 등록된다
- [ ] **ADOBE-02**: PSD 썸네일이 30초 안에 생성되어 프로젝트 상세 화면에 표시된다 (native QL → qlmanage → embedded JPEG → placeholder 폴백)
- [ ] **ADOBE-03**: 파일명 패턴(시안 v\d, sample, _v\d, seq\d)으로 시안 버전을 자동 인식한다
- [ ] **ADOBE-04**: AI/INDD 같이 썸네일 생성 실패 가능 포맷도 placeholder + "외부에서 보기"로 graceful degradation한다

### KakaoTalk Ingest (KAKAO)

- [ ] **KAKAO-01**: 사용자가 NSOpenPanel로 와치 폴더를 지정한다 (Full Disk Access 불요)
- [ ] **KAKAO-02**: 폴더에 .txt 파일이 추가/수정되면 size-stable debounce(2s) 후 처리된다 (file-being-written 상태 회피)
- [ ] **KAKAO-03**: KR PC / KR Mobile / EN PC / ZH PC 4가지 로케일 포맷을 자동 감지·파싱한다
- [ ] **KAKAO-04**: UTF-8 BOM과 CRLF/LF 차이가 정상 처리된다
- [ ] **KAKAO-05**: 같은 .txt를 두 번 떨어뜨려도 SHA256(timestamp, sender, body) idempotency로 중복 등록되지 않는다
- [ ] **KAKAO-06**: 처음 본 대화방 이름은 "어떤 프로젝트?" 매핑 마법사를 띄우고 결과를 저장해 다음부터 자동 매핑한다
- [ ] **KAKAO-07**: AI가 .txt에서 결정·일정·요청을 추출해 노션 페이지에 1분 안에 반영한다
- [ ] **KAKAO-08**: 지원되지 않는 로케일은 명확한 에러 메시지를 띄우고 사용자에게 fixture 제공을 요청한다

### AI Daily Brief (BRIEF)

- [ ] **BRIEF-01**: 매일 사용자 지정 시간(기본 8:00)에 macOS 알림이 발송된다
- [ ] **BRIEF-02**: 알림 클릭 시 5초 안에 오늘의 대시보드가 열린다
- [ ] **BRIEF-03**: launchd LaunchAgent는 RunAtLoad=true만 사용하고 8시 트리거는 앱 내 tokio 태스크가 수행한다 (StartCalendarInterval 금지)
- [ ] **BRIEF-04**: macOS sleep으로 8시를 놓치면 NSWorkspaceDidWakeNotification로 wake 시 미발송분을 감지해 즉시 발송한다
- [ ] **BRIEF-05**: 브리프는 모든 채널의 오늘 이벤트를 종합해 "오늘 액션 N건, 가장 급한 일: X" 형식으로 표시한다
- [ ] **BRIEF-06**: AI 인사이트(예: "샘플 사이클이 평균보다 늦음")는 프로젝트 페이지 주석으로만 표시되고 진행률 점수에는 반영되지 않는다
- [ ] **BRIEF-07**: 브리프에 표시된 액션 70%↑가 실제 그날 처리됨 (4주 운영 후 자체 측정)

### Today View / Snooze / Roll-over (TODAY)

- [ ] **TODAY-01**: 메인 윈도우 좌측의 "오늘 할 일" 리스트는 모든 프로젝트의 오늘 액션을 통합 표시한다
- [ ] **TODAY-02**: 각 액션을 [완료/스누즈 N분/내일로 이월/주말로 이월/취소]할 수 있다
- [ ] **TODAY-03**: 처리 안 한 항목이 일주일 뒤에도 그대로 쌓이지 않게 일별 자동 roll-over 정책을 가진다
- [ ] **TODAY-04**: 진행률 바는 마일스톤 60% + 태스크 40% 가중 평균으로 계산된다

### Cross-channel Search (SEARCH)

- [ ] **SEARCH-01**: SQLite FTS5로 모든 채널의 흡수된 메시지를 로컬 검색할 수 있다
- [ ] **SEARCH-02**: 1만 건 메시지에서 검색 결과가 500ms 안에 반환된다
- [ ] **SEARCH-03**: 검색 결과에 채널·프로젝트·날짜·발신자 메타데이터가 포함된다
- [ ] **SEARCH-04**: 검색 결과 클릭 시 해당 메시지를 원본 채널에서 열 수 있는 deep link를 제공한다 (가능 채널: Slack/Notion/Gmail)

### Send Workflow (SEND)

- [ ] **SEND-01**: AI가 프로젝트 컨텍스트를 보고 발송할 메시지 후보 카드를 생성한다
- [ ] **SEND-02**: 후보 카드에 [수정/발송/무시/다른 채널로] 버튼이 있다
- [ ] **SEND-03**: AI 출력은 structured JSON이며 recipient_id/project_id/milestone_id/sample_id가 모두 SQLite의 실제 ID를 참조해야 한다 (조작된 ID는 거부)
- [ ] **SEND-04**: 수신자 결정은 결정론적 코드 — AI는 절대 to: 필드를 채울 수 없다
- [ ] **SEND-05**: 발송 직전 확인 화면에 수신자 이름·채널·다른 프로젝트 연관 수·수신자 로컬 시간이 표시된다
- [ ] **SEND-06**: 수신자 로컬 22:00–06:00 발송은 추가 confirm을 요구한다
- [ ] **SEND-07**: AI 초안은 수신자 역할(MD/공장/내부)에 따라 존댓말/반말/영문을 자동 선택한다
- [ ] **SEND-08**: 같은 프로젝트 컨텍스트만 AI에 전달 — 다른 프로젝트의 정보가 cross-contaminate되지 않는다
- [ ] **SEND-09**: 발송 즉시 노션 프로젝트의 Communication Log에 자동 기록된다
- [ ] **SEND-10**: AI 본문 vs 최종 발송 본문 diff가 logged되어 주간 "AI changed N facts" 메트릭으로 보인다
- [ ] **SEND-11**: 발송 채널은 Slack(채널/DM)과 Gmail을 모두 지원한다
- [ ] **SEND-12**: AI 후보 채택률(발송 또는 수정 후 발송) ≥ 50% (4주 측정)

### Read-only Share URL (SHARE)

- [ ] **SHARE-01**: 토큰화된 공유 URL을 무료 Cloudflare `*.workers.dev` 서브도메인에서 발급한다 (예: `designer-dashboard.workers.dev/p/{slug}/{token}`)
- [ ] **SHARE-02**: 공유 페이지에는 진행률·일정·다음 단계만 표시되고 메시지 본문/이메일 제목/스니펫은 절대 노출되지 않는다 (CI test가 forbidden field 검증)
- [ ] **SHARE-03**: SharedView Rust 타입은 구조적으로 body/preview/snippet 필드를 가지지 않아 컴파일 시점에 누설을 차단한다
- [ ] **SHARE-04**: 토큰 회수 시 5초 안에 다음 요청부터 403을 반환한다
- [ ] **SHARE-05**: 공유 토큰 기본 만료는 7일이며 사용자가 1-click으로 연장할 수 있다
- [ ] **SHARE-06**: 프로젝트마다 confidentiality tier(internal/shared/public)를 설정할 수 있고 internal은 공유 URL 발급이 차단된다
- [ ] **SHARE-07**: 공유 스냅샷에 단가/원가/마진/경쟁사명 같은 민감어가 포함되면 사전 경고를 띄운다
- [ ] **SHARE-08**: 외부 모바일에서 공유 URL이 5초 안에 로드된다

### Privacy & Consent (PRIV)

- [ ] **PRIV-01**: 첫 실행 시 "AI에 메시지를 보내는 것에 동의" 화면이 명시적으로 떠야 한다
- [ ] **PRIV-02**: 사용자가 채널별·DB별로 AI 처리를 끌 수 있다
- [ ] **PRIV-03**: 메시지 본문은 로컬 SQLite에만 저장되고 Cloudflare KV로 절대 전송되지 않는다 (네트워크 캡처로 검증 가능)
- [ ] **PRIV-04**: AI 호출은 Anthropic API만 사용하고 Anthropic은 학습 미사용 정책이라는 사실을 설정 화면에 명시한다
- [ ] **PRIV-05**: 사용자가 "전체 데이터 삭제" 버튼으로 로컬 + 모든 공유 KV 토큰을 일괄 삭제할 수 있다

### AI Cost Controls (COST)

- [ ] **COST-01**: 일일 AI 예산 상한이 SQLite에 저장되고 모든 AI 호출 전에 체크된다 (기본 $1/일)
- [ ] **COST-02**: 예산 초과 시 추가 호출은 차단되고 사용자에게 알림이 표시된다
- [ ] **COST-03**: 같은 (model, system_prompt, user_message) 호출은 7일간 응답 캐시로 재사용된다
- [ ] **COST-04**: 항목별 last_ai_run_at + ai_run_count 추적으로 24시간 내 3회 초과 시 freeze한다 (피드백 루프 방지)
- [ ] **COST-05**: 우리 통합이 만든 노션 변경은 last_edited_by 태그로 식별되어 reconciliation에서 스킵된다 (loop-breaker)
- [ ] **COST-06**: 분류 경로는 코드 레벨에서 Haiku 4.5만 호출 가능 (Sonnet 호출 금지)
- [ ] **COST-07**: 프롬프트 캐시 히트율 ≥ 70% (프로덕션 telemetry로 측정)

### Onboarding (ONBOARD)

- [ ] **ONBOARD-01**: 첫 실행 5단계 마법사: Notion → Google → Slack → KakaoTalk 폴더 → 알림 시간
- [ ] **ONBOARD-02**: 각 단계는 "나중에 연결" 옵션이 있어 차단 없이 진행 가능하다
- [ ] **ONBOARD-03**: 각 단계 완료/skip이 명확히 표시되고 설정에서 다시 진입할 수 있다
- [ ] **ONBOARD-04**: 완주율 90%↑ (베타 5명 기준)

### Distribution (DIST)

- [ ] **DIST-01**: Apple Developer ID로 코드 서명되고 `xcrun notarytool`으로 notarize된 DMG로 배포된다
- [ ] **DIST-02**: Hardened Runtime이 켜져 있고 필수 entitlement가 정확히 설정된다
- [ ] **DIST-03**: `tauri-plugin-updater`가 ed25519 서명된 manifest를 R2에서 가져와 자동 업데이트한다
- [ ] **DIST-04**: CI가 매 PR마다 notarization smoke test(`xcrun notarytool submit --wait`)를 실행하고 실패 시 merge를 차단한다
- [ ] **DIST-05**: CFBundleVersion이 매 릴리스마다 증가한다

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Beyond v1

- **V2-SHUTDOWN**: 하루 마감 의식(Sunsama 패턴) — 오늘 회고 + 내일 미리보기
- **V2-CMDK**: Cmd+K 명령 팔레트 — 모든 액션 빠른 호출
- **V2-RECUR**: 반복 태스크 템플릿 ("매주 월요일 공장 진행 보고")
- **V2-EXPORT**: CSV/PDF export
- **V2-MULTI-LANG**: 발송 메시지 한/中/英 자동 감지 및 다국어 초안
- **V2-PER-PROJ-MILE**: 프로젝트마다 마일스톤 템플릿 커스터마이즈 (v1은 단일 12단계)
- **V2-MULTI-WS**: 다중 슬랙/구글 워크스페이스
- **V2-ADOBE-API**: Adobe Creative Cloud Libraries API 직접 연동
- **V2-OS**: Windows / Linux 데스크탑
- **V2-MOBILE**: iOS/Android 컴패니언 앱
- **V2-TEAM**: MD/공장 단방향 R 외에 양방향 팀 모드
- **V2-NOTION-WEBHOOK**: Notion webhooks를 polling 보조 → 일차 sync로 승격
- **V2-AI-CHANNEL-SUGGEST**: 새 슬랙 채널 자동 분류 학습
- **V2-VOICE**: 음성-to-task
- **V2-INSIGHT-COMMENT**: AI 인사이트를 노션 페이지 코멘트로 자동 게시
- **V2-JANDI-SWIT**: JANDI/Swit 등 한국 협업 도구 연동
- **V2-BYOD**: Bring Your Own Domain (커스텀 공유 도메인)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| 카카오톡 공식 API 직접 연동 | 개인 톡 API 부재, 비공식 자동화는 계정 정지 위험. .txt 우회만 v1에서 지원 |
| Adobe Creative Cloud Libraries API 직접 호출 | OAuth 무게 큼 + v1 가치 낮음. 슬랙 첨부 인지로 충분 |
| Windows / Linux 데스크탑 (v1) | macOS 단일 OS로 v1 범위 한정. Tauri 구조로 후속 OS 가능 |
| MD/공장용 쓰기 권한 | 외부 인증·권한 인프라 v1에서 비대 → 공유 URL R-only로 충분 |
| 자동 발송(승인 없이) — 자유 메시지 | 디자이너 톤·뉘앙스 실수 위험. 정형 알림(D-3 리마인드)만 옵트인 자동 |
| 모바일 컴패니언 앱 | v1은 데스크탑 전용. 모바일은 공유 URL을 브라우저로 본다 |
| 다중 워크스페이스 (구글/슬랙) | v1은 단일 계정 |
| 한국어 외 1차 지원 | UI는 한국어, 발송만 영문 자동 감지 정도 |
| 유료 커스텀 도메인 | v1은 무료 *.workers.dev. BYOD는 v2 |
| Sparkle 프레임워크 (구버전) | 단일 메인테이너 + 5MB 추가. 공식 tauri-plugin-updater가 더 안전 |

## Traceability

Phase mapping for every v1 REQ-ID. Phases derive from research/SUMMARY.md sequencing (P0 Foundation → P8 Distribution + Notion webhooks). See ROADMAP.md for full phase definitions and goal-backward success criteria.

| Requirement | Phase | Status |
|-------------|-------|--------|
| FOUND-01 | P0 — Foundation Shell | Pending |
| FOUND-02 | P0 — Foundation Shell | Pending |
| FOUND-03 | P0 — Foundation Shell | Pending |
| FOUND-04 | P0 — Foundation Shell | Pending |
| FOUND-05 | P0 — Foundation Shell | Pending |
| FOUND-06 | P0 — Foundation Shell | Pending |
| NOTION-01 | P1 — Notion SOTR + Schema Wizard + LWW | Pending |
| NOTION-02 | P1 — Notion SOTR + Schema Wizard + LWW | Pending |
| NOTION-03 | P1 — Notion SOTR + Schema Wizard + LWW | Pending |
| NOTION-04 | P1 — Notion SOTR + Schema Wizard + LWW | Pending |
| NOTION-05 | P1 — Notion SOTR + Schema Wizard + LWW | Pending |
| NOTION-06 | P1 — Notion SOTR + Schema Wizard + LWW | Pending |
| NOTION-07 | P1 — Notion SOTR + Schema Wizard + LWW | Pending |
| NOTION-08 | P1 — Notion SOTR + Schema Wizard + LWW | Pending |
| NOTION-09 | P0 — Foundation Shell (rate-limiter primitive) | Pending |
| GCAL-01 | P4 — Gmail + Google Calendar | Pending |
| GCAL-02 | P4 — Gmail + Google Calendar | Pending |
| GCAL-03 | P4 — Gmail + Google Calendar | Pending |
| GCAL-04 | P4 — Gmail + Google Calendar | Pending |
| GMAIL-01 | P4 — Gmail + Google Calendar | Pending |
| GMAIL-02 | P4 — Gmail + Google Calendar | Pending |
| GMAIL-03 | P4 — Gmail + Google Calendar | Pending |
| GMAIL-04 | P4 — Gmail + Google Calendar | Pending |
| GMAIL-05 | P4 — Gmail + Google Calendar | Pending |
| GMAIL-06 | P4 — Gmail + Google Calendar | Pending |
| SLACK-01 | P3 — Slack + AI Classifier | Pending |
| SLACK-02 | P3 — Slack + AI Classifier | Pending |
| SLACK-03 | P3 — Slack + AI Classifier | Pending |
| SLACK-04 | P3 — Slack + AI Classifier | Pending |
| SLACK-05 | P3 — Slack + AI Classifier | Pending |
| SLACK-06 | P3 — Slack + AI Classifier | Pending |
| SLACK-07 | P3 — Slack + AI Classifier | Pending |
| ADOBE-01 | P5 — KakaoTalk + Adobe | Pending |
| ADOBE-02 | P5 — KakaoTalk + Adobe | Pending |
| ADOBE-03 | P5 — KakaoTalk + Adobe | Pending |
| ADOBE-04 | P5 — KakaoTalk + Adobe | Pending |
| KAKAO-01 | P5 — KakaoTalk + Adobe | Pending |
| KAKAO-02 | P5 — KakaoTalk + Adobe | Pending |
| KAKAO-03 | P5 — KakaoTalk + Adobe | Pending |
| KAKAO-04 | P5 — KakaoTalk + Adobe | Pending |
| KAKAO-05 | P5 — KakaoTalk + Adobe | Pending |
| KAKAO-06 | P5 — KakaoTalk + Adobe | Pending |
| KAKAO-07 | P5 — KakaoTalk + Adobe | Pending |
| KAKAO-08 | P5 — KakaoTalk + Adobe | Pending |
| BRIEF-01 | P6 — AI Brief + Today-View + Search | Pending |
| BRIEF-02 | P6 — AI Brief + Today-View + Search | Pending |
| BRIEF-03 | P6 — AI Brief + Today-View + Search | Pending |
| BRIEF-04 | P6 — AI Brief + Today-View + Search | Pending |
| BRIEF-05 | P6 — AI Brief + Today-View + Search | Pending |
| BRIEF-06 | P6 — AI Brief + Today-View + Search | Pending |
| BRIEF-07 | P6 — AI Brief + Today-View + Search | Pending |
| TODAY-01 | P6 — AI Brief + Today-View + Search | Pending |
| TODAY-02 | P6 — AI Brief + Today-View + Search | Pending |
| TODAY-03 | P6 — AI Brief + Today-View + Search | Pending |
| TODAY-04 | P1 — Notion SOTR (progress calc lives with milestones/tasks) | Pending |
| SEARCH-01 | P6 — AI Brief + Today-View + Search | Pending |
| SEARCH-02 | P6 — AI Brief + Today-View + Search | Pending |
| SEARCH-03 | P6 — AI Brief + Today-View + Search | Pending |
| SEARCH-04 | P6 — AI Brief + Today-View + Search | Pending |
| SEND-01 | P7 — Send Cards + Approval Gate | Pending |
| SEND-02 | P7 — Send Cards + Approval Gate | Pending |
| SEND-03 | P7 — Send Cards + Approval Gate | Pending |
| SEND-04 | P7 — Send Cards + Approval Gate | Pending |
| SEND-05 | P7 — Send Cards + Approval Gate | Pending |
| SEND-06 | P7 — Send Cards + Approval Gate | Pending |
| SEND-07 | P7 — Send Cards + Approval Gate | Pending |
| SEND-08 | P7 — Send Cards + Approval Gate | Pending |
| SEND-09 | P7 — Send Cards + Approval Gate | Pending |
| SEND-10 | P7 — Send Cards + Approval Gate | Pending |
| SEND-11 | P7 — Send Cards + Approval Gate | Pending |
| SEND-12 | P7 — Send Cards + Approval Gate | Pending |
| SHARE-01 | P2 — Cloudflare Bridge + Share Skeleton + SharedView | Pending |
| SHARE-02 | P2 — Cloudflare Bridge + Share Skeleton + SharedView | Pending |
| SHARE-03 | P2 — Cloudflare Bridge + Share Skeleton + SharedView | Pending |
| SHARE-04 | P2 — Cloudflare Bridge + Share Skeleton + SharedView | Pending |
| SHARE-05 | P2 — Cloudflare Bridge + Share Skeleton + SharedView | Pending |
| SHARE-06 | P2 — Cloudflare Bridge + Share Skeleton + SharedView | Pending |
| SHARE-07 | P2 — Cloudflare Bridge + Share Skeleton + SharedView | Pending |
| SHARE-08 | P2 — Cloudflare Bridge + Share Skeleton + SharedView | Pending |
| PRIV-01 | P3 — Slack + AI Classifier (first AI consent gate) | Pending |
| PRIV-02 | P3 — Slack + AI Classifier (per-channel AI-off) | Pending |
| PRIV-03 | P2 — SharedView privacy boundary | Pending |
| PRIV-04 | P3 — Slack + AI Classifier (Anthropic policy disclosure) | Pending |
| PRIV-05 | P2 — SharedView (KV token bulk-delete) | Pending |
| COST-01 | P0 — Foundation Shell (AI budget primitive) | Pending |
| COST-02 | P0 — Foundation Shell (budget enforcement) | Pending |
| COST-03 | P3 — Slack + AI Classifier (idempotency cache exercised) | Pending |
| COST-04 | P3 — Slack + AI Classifier (loop detection) | Pending |
| COST-05 | P1 — Notion SOTR (last_edited_by loop-breaker) | Pending |
| COST-06 | P3 — Slack + AI Classifier (Haiku-only compile guard) | Pending |
| COST-07 | P3 — Slack + AI Classifier (cache hit telemetry) | Pending |
| ONBOARD-01 | P8 — Polish + Distribution + Notion Webhooks | Pending |
| ONBOARD-02 | P8 — Polish + Distribution + Notion Webhooks | Pending |
| ONBOARD-03 | P8 — Polish + Distribution + Notion Webhooks | Pending |
| ONBOARD-04 | P8 — Polish + Distribution + Notion Webhooks | Pending |
| DIST-01 | P8 — Polish + Distribution + Notion Webhooks | Pending |
| DIST-02 | P0 — Foundation Shell (Hardened Runtime baseline) | Pending |
| DIST-03 | P8 — Polish + Distribution + Notion Webhooks | Pending |
| DIST-04 | P8 — Polish + Distribution + Notion Webhooks | Pending |
| DIST-05 | P8 — Polish + Distribution + Notion Webhooks | Pending |

**Coverage:**
- v1 requirements: 100 total (FOUND 6 + NOTION 9 + GCAL 4 + GMAIL 6 + SLACK 7 + ADOBE 4 + KAKAO 8 + BRIEF 7 + TODAY 4 + SEARCH 4 + SEND 12 + SHARE 8 + PRIV 5 + COST 7 + ONBOARD 4 + DIST 5)
- Mapped to phases: 100 (P0: 10, P1: 10, P2: 10, P3: 14, P4: 10, P5: 12, P6: 14, P7: 12, P8: 8) — sum 100 ✓
- Unmapped: 0

**Per-phase REQ counts** (sanity check):
- P0 = 6 FOUND + NOTION-09 + COST-01 + COST-02 + DIST-02 = 10
- P1 = NOTION-01..08 + TODAY-04 + COST-05 = 10
- P2 = 8 SHARE + PRIV-03 + PRIV-05 = 10
- P3 = 7 SLACK + COST-03,04,06,07 + PRIV-01,02,04 = 14
- P4 = 6 GMAIL + 4 GCAL = 10
- P5 = 8 KAKAO + 4 ADOBE = 12
- P6 = 7 BRIEF + TODAY-01,02,03 + 4 SEARCH = 14
- P7 = 12 SEND = 12
- P8 = 4 ONBOARD + DIST-01,03,04,05 = 8

---
*Requirements defined: 2026-05-03*
*Last updated: 2026-05-03 with phase traceability after roadmap creation*
