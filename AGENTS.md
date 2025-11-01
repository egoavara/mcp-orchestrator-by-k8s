# 프로젝트 핵심 목표

## 핵심 유의사항
- 모든 파일은 가능한 300줄 이하로 관리한다
- 개념적으로 이해하기 쉽도록 코드를 분리한다
- 개발할 때는 release 빌드를 하지 말 것, release 빌드는 사용자가 요구하는 경우에만 수행한다.
- trunk serve는 사용자만 수행한다
- crate 버전은 각 crate 보다는 workspace 레벨에 정의하고 subcrate 에서는 `{workspace=true}` 로 사용한다
- 유사한 역활의 라이브러리를 사용하지 않는다, `ex. time + chrono 중 먼저 사용중인 라이브러리로 최대한 활용`

## 주요 기능
- stdio 기반 MCP 서버를 HTTP-SSE 스트림으로 변환
- HTTP API로 MCP 서버 생성/삭제/관리
- Kubernetes로 각 MCP 서버 인스턴스 오케스트레이션

## 아키텍처
- **Backend**: Rust 기반 HTTP API 서버
- **MCP 관리**: K8s Pod로 stdio MCP 서버 배포
- **통신**: stdio ↔ HTTP-SSE 브릿지

## 리소스간 의존 그래프 참조
- spec/DEPENDENCY.md 에 존재

## 핵심 컴포넌트
1. HTTP API: MCP 서버 CRUD
2. K8s Controller: Pod 생성/관리/삭제
3. Protocol Bridge: stdio ↔ HTTP-SSE 변환
4. Session 관리: 클라이언트별 MCP 인스턴스 매핑

## Active Technologies
- Rust 1.90.0 + kube 0.x, k8s-openapi 0.x, prost (protobuf), serde_json, tonic (002-resource-limit-scheduling)
- Kubernetes ConfigMaps (JSON data fields) (002-resource-limit-scheduling)

## Recent Changes
- 002-resource-limit-scheduling: Added Rust 1.90.0 + kube 0.x, k8s-openapi 0.x, prost (protobuf), serde_json, tonic
