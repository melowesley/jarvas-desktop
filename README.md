# Jarvas Desktop

Cliente desktop nativo do **Jarvas** para Windows — um *thin client* (Tauri 2 +
WebView2) que abre a UI web já existente em `https://jarvas.agendavet.vet.br` numa
janela própria, sem barra de endereço nem abas.

> Tarefa 8 (fase 2) do projeto JOOD. Design completo: `jood/specs/2026-06-22-tauri-desktop-design.md`
> no repo do DeerFlow. Responder sempre em PT-BR.

## Como funciona

- A **UI é remota**. Toda lógica nativa roda no lado **Rust** (`src-tauri/src/lib.rs`);
  a página remota é uma webview comum, **sem acesso ao IPC do Tauri**.
- **Startup**: um connect TCP curto (3s) decide entre carregar a URL remota ou a tela
  local `dist/offline.html` (sem conexão → botão "Tentar de novo").
- **Links externos** (host ≠ `jarvas.agendavet.vet.br`) abrem no navegador do sistema.
- **Auto-update**: plugin updater do Tauri; manifesto (`latest.json`) + binários em
  GitHub Releases públicos. Falhas de update são ignoradas silenciosamente.
- **Sessão**: o WebView2 persiste cookies/localStorage → login email/senha sobrevive
  entre aberturas.

## Estrutura

```
jarvas-desktop/
├─ dist/                      # frontendDist (exigido pelo Tauri)
│  ├─ index.html              # placeholder, não usado em runtime
│  └─ offline.html            # tela "sem conexão" (carregada via WebviewUrl::App)
├─ src-tauri/
│  ├─ tauri.conf.json         # janela, updater, bundle NSIS, webview2 bootstrapper
│  ├─ Cargo.toml
│  ├─ build.rs
│  ├─ capabilities/default.json   # nenhuma permissão de IPC exposta
│  ├─ icons/                  # icon.ico + PNGs (gerados da logo mosaico Jarvas)
│  └─ src/{main.rs, lib.rs}
└─ .github/workflows/release.yml  # tag v* → build Windows → GitHub Release
```

> **Nota de design:** a `offline.html` fica em `dist/` (frontendDist), não em
> `src-tauri/resources/`, para ser carregável na webview via `WebviewUrl::App`
> (a spec previa essa decisão em §13).

## Build / release

Não é preciso instalar Rust localmente — o build roda no **GitHub Actions**
(`windows-latest`). Para publicar uma versão:

1. Bump da `version` em `src-tauri/tauri.conf.json`.
2. `git tag v0.1.0 && git push origin v0.1.0` (o push é feito pelo Wesley).
3. O CI builda o `.exe` (NSIS), assina o pacote de update e cria o GitHub Release
   com o instalador + `latest.json`.

### Pré-requisitos de uma vez (segredos)

- Par de chaves do **updater** (`npx @tauri-apps/cli signer generate`):
  - **pública** → `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`).
  - **privada** + senha → secrets do Actions `TAURI_SIGNING_PRIVATE_KEY` e
    `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. **Nunca** commitar a privada.

> O `pubkey` atual em `tauri.conf.json` é `UPDATER_PUBKEY_PLACEHOLDER` — substituir
> pela chave pública real antes do primeiro release.

## Testes

Funções puras testáveis (`cargo test`, roda no CI): `is_internal()` (host interno vs.
externo, incl. casos maliciosos de sufixo) e `update_endpoint()` (formato do
`latest.json`).

## Fora de escopo (fase 2)

macOS/Linux · code signing (Authenticode → SmartScreen) · OAuth no app · tray/atalho
global/deep links · modo offline real.
