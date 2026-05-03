# Designer Dashboard

## What This Is

제품 개발 디자이너가 매일 아침 5분 안에 오늘 할 일과 보낼 메시지를 한눈에 확인할 수 있게 해주는 macOS 메뉴바 앱이다. 슬랙·지메일·구글 캘린더·노션·카카오톡(.txt 내보내기)에서 흩어진 프로젝트 정보를 흡수해 노션을 단일 진실의 원천으로 통합하고, AI가 매일 브리프와 MD/공장에 보낼 지시 초안을 생성해 사용자가 검토 후 발송한다. MD와 공장에는 읽기전용 공유 URL로 진행 상황만 노출한다.

## Core Value

매일 아침 5분 안에, 오늘의 액션 목록과 줄 지시 초안을 본다.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] **Foundation**: macOS 메뉴바 트레이 앱이 실행되고 OS 부팅 시 자동 시작된다
- [ ] **Foundation**: OAuth 토큰을 macOS Keychain에 저장하고 재시작 후에도 사용 가능하다
- [ ] **Notion (SOTR)**: 노션 프로젝트 DB와 양방향 동기화 — 노션 변경이 5분 내 앱에 반영, 앱 변경이 즉시 노션에 반영된다
- [ ] **Notion (SOTR)**: 노션 DB 스키마 마법사가 부족한 속성을 자동 보강한다
- [ ] **Calendar**: 구글 캘린더 이벤트가 양방향 동기화되고 마일스톤 due_date 변경이 캘린더에 자동 반영된다
- [ ] **Gmail**: 라벨링된 메일이 100% 자동으로 프로젝트에 분류된다
- [ ] **Slack**: 슬랙 메시지가 채널명/본문/AI 분류로 90%↑ 정확도로 프로젝트에 매핑된다
- [ ] **Slack/Adobe**: 슬랙에 업로드된 .psd/.ai/.indd/시안 이미지가 30초 안에 프로젝트 갤러리에 썸네일로 나타난다
- [ ] **KakaoTalk**: 와치 폴더에 .txt 내보내기 파일이 떨어지면 1분 안에 노션에 정보가 반영된다 (중복 방지 포함)
- [ ] **AI Brief**: 매일 8시 macOS 알림으로 오늘의 액션을 푸시하고 클릭 시 5초 안에 대시보드가 열린다
- [ ] **AI Brief**: AI 인사이트가 프로젝트 페이지에 주석으로 첨부된다 (점수에는 반영 X)
- [ ] **Send**: AI가 만든 지시 초안 카드를 본인이 [수정/발송/무시]로 처리하고 발송 결과가 노션 Communication Log에 자동 기록된다
- [ ] **Send**: 발송 채널은 슬랙(채널/DM)과 지메일을 모두 지원한다
- [ ] **Share**: 무료 Cloudflare Workers 서브도메인(`designer-dashboard.workers.dev`)에서 토큰화된 읽기전용 공유 URL을 발급/만료/회수할 수 있고 회수 시 즉시 403을 반환한다 (커스텀 도메인은 v2 옵션)
- [ ] **Share**: 공유 페이지에는 메시지 본문이 절대 노출되지 않는다 (진행률·일정·다음 단계만)
- [ ] **Progress**: 프로젝트 진행률이 마일스톤(60%) + 태스크(40%) 가중 평균으로 계산된다
- [ ] **Privacy**: 메시지 본문은 로컬 SQLite에만 저장되고 클라우드(공유 KV)에 절대 푸시되지 않는다
- [ ] **Onboarding**: 첫 실행 5단계 마법사로 모든 채널 인증 + 와치 폴더 + 알림 시간을 설정할 수 있다
- [ ] **Distribution**: 코드 서명된 DMG로 배포되고 Sparkle을 통해 자동 업데이트된다

### Out of Scope

- 카카오톡 공식 API 직접 연동 — 개인 톡방 API 부재 + 비공식 자동화는 계정 정지 위험. v1은 .txt 내보내기 우회만 지원
- Adobe Creative Cloud Libraries API 직접 호출 — v1은 슬랙 첨부 이미지 인지로 충분, 직접 연동은 v2
- Windows / Linux 데스크탑 — v1은 macOS 전용. 구조는 Tauri 기반이라 후속 OS 확장 가능하지만 v1 범위 아님
- MD/공장용 쓰기 권한 — v1은 본인 단독 R/W, 외부는 공유 URL 읽기전용만. 외부 인증·권한 인프라는 v1 범위 초과
- 자동 발송(승인 없이) — 디자이너 톤·뉘앙스 오류 위험으로 모든 발송은 사용자 승인 게이트 필수
- 모바일 컴패니언 앱 — v1은 macOS 데스크탑만. 모바일은 공유 URL을 브라우저로 보는 방식
- 다중 워크스페이스 (구글/슬랙) — v1은 단일 계정. 다중 계정은 v2
- 유료 커스텀 도메인 — v1은 무료 `*.workers.dev` 서브도메인만. BYOD(Bring Your Own Domain)는 v2 옵션
- 한국어 외 1차 지원 — UI는 한국어 우선, 영어/중국어는 발송 메시지 자동 감지 정도만

## Context

**사용자**: 제품 개발 디자이너 (제조업 — 패션/리빙/잡화 등). 시안 디자인 → 샘플 발주 → 검수 → 양산 → 출고까지 전 과정의 일정·품질을 책임짐. MD(머천다이저)와 제조 공장(국내/해외)이 주요 협업 대상.

**현재 도구 환경**: 클로드, 슬랙, 카카오톡, 지메일, 노션, 구글 캘린더, 어도비 (PSD/AI/INDD). 매일 4~5개 채널을 떠돌며 정보 추적, 평균 30분~1시간 소요. MD·공장이 보는 정보와 본인이 보는 정보가 자주 어긋남.

**도메인 라이프사이클** (마일스톤 템플릿 12단계):
기획 → 1차 시안 → 1차 피드백 → 2차/확정 → 샘플 발주 → 샘플 도착 → (샘플 수정) → 컨펌 → 양산 발주 → 양산 진행 → QC/QA → 출고/입고

**기술 환경**:
- 셸: Tauri 2.x (Rust + React/TypeScript), 번들 30~50MB
- UI: React 18 + Tailwind + shadcn/ui
- 로컬: SQLite + macOS Keychain
- AI: Anthropic SDK, Sonnet 4.6 + Haiku 4.5, prompt caching 기본
- 공유: Cloudflare Workers + KV + R2 (sanitized 스냅샷만)

**예상 비용**: AI 호출 ~$10/월 (balanced 모드, 캐싱 적용). Cloudflare 무료 한도 내.

## Constraints

- **Tech stack**: Tauri 2.x + React + Rust — Electron 대비 메모리 1/4, 디자이너 사용자의 메모리 민감도 고려
- **Privacy**: 메시지 본문은 절대 클라우드 외부로 나가지 않음(AI 호출 제외, Anthropic 무학습) — 사용자가 다루는 정보가 미공개 디자인·발주처 정보를 포함
- **Source of Truth**: 노션이 SOTR — 다른 채널 데이터는 흡수해 노션에 기록, 충돌 시 노션 우선 (Last-Write-Wins by `last_edited_time`)
- **KakaoTalk**: 공식 API로 개인 톡 읽기/쓰기 불가 → 데스크탑 .txt 내보내기 + 와치 폴더 우회만 사용
- **Send policy**: 모든 외부 발송은 사용자 승인 게이트 통과 필수 — 자동 발송은 정형 알림(D-3 리마인드 등)에 한해 옵트인
- **Platform**: macOS only (v1) — Tauri 구조상 후속 OS 추가는 가능하지만 v1 범위 외
- **Distribution**: 코드 서명 + 자동 업데이트 — 사용자가 Gatekeeper 차단/수동 업데이트로 이탈하지 않게
- **Cost**: v1 운영비 0원 인프라 — Cloudflare Workers/KV/R2 무료 한도, `*.workers.dev` 서브도메인 사용. 도메인 등록 비용 없음. AI 호출 ~$10/월 + Apple Developer ID $99/년만 발생

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| 플랫폼: macOS 메뉴바 (Tauri 2.x) | "데스크탑에 항상 떠있는 아이콘" 요건 + 메모리 민감 사용자 + 후속 OS 확장 가능 | — Pending |
| 카카오톡: .txt 내보내기 + 와치 폴더 | 공식 API 부재, 비공식 자동화는 계정 정지 위험. 반자동이 안전·합법 한계 | — Pending |
| 발송 정책: AI 초안 → 본인 승인 → 발송 | 디자이너 톤·뉘앙스 실수 방지, 5초 검토는 큰 안전장치 | — Pending |
| 사용자: 본인 + 읽기전용 공유 URL | 외부 인증·권한 인프라 v1에서 비대 → 공유 URL이 80% 가치 달성 | — Pending |
| 진행률: 마일스톤 60% + 태스크 40% + AI 주석 별도 | AI 추론은 틀릴 수 있어 점수 미반영, 명시적 체크포인트가 신뢰 가능 | — Pending |
| Adobe: v1은 슬랙 첨부 인지만 | Creative Cloud Libraries API는 OAuth 무게 큰데 v1 가치 낮음 | — Pending |
| 데이터: 로컬 SQLite + 공유용만 Cloudflare KV | 프라이버시 + 비용 모두 유리, 메시지 본문은 절대 외부 X | — Pending |
| SOTR: 노션 | 사용자 이미 노션 사용 중, 4채널 통합 시 충돌 해소를 한 곳에서 | — Pending |
| AI: Anthropic Sonnet 4.6 + Haiku 4.5 + 프롬프트 캐싱 | 한국어 최상위, 사용자 친숙, 캐싱으로 ~$10/월 | — Pending |
| 공유 호스팅: Cloudflare Workers + KV + R2, `*.workers.dev` 무료 서브도메인 | 무료 한도 내·도메인 등록 비용 0원·정적·빠름·토큰 회수 즉시 반영 | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-03 after initialization*
