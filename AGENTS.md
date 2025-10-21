# 프로젝트 핵심 목표

## 주요 기능
- stdio 기반 MCP 서버를 HTTP-SSE 스트림으로 변환
- HTTP API로 MCP 서버 생성/삭제/관리
- Kubernetes로 각 MCP 서버 인스턴스 오케스트레이션

## 아키텍처
- **Backend**: Rust 기반 HTTP API 서버
- **MCP 관리**: K8s Pod로 stdio MCP 서버 배포
- **통신**: stdio ↔ HTTP-SSE 브릿지

## 핵심 컴포넌트
1. HTTP API: MCP 서버 CRUD
2. K8s Controller: Pod 생성/관리/삭제
3. Protocol Bridge: stdio ↔ HTTP-SSE 변환
4. Session 관리: 클라이언트별 MCP 인스턴스 매핑
