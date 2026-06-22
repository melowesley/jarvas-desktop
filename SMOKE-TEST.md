# Smoke test — Jarvas Desktop

Verificação manual após o CI publicar o release (a lógica pura já é coberta por
`cargo test` no CI). Marque cada item ao validar.

## 0. Pré-condições (CI verde)
- [ ] Secret `TAURI_SIGNING_PRIVATE_KEY` cadastrado (conteúdo de `~/.tauri/jarvas-desktop.key`).
- [ ] Secret antigo `TAURI_PRIVATE_KEY` (PEM) **removido**.
- [ ] Tag `v*` empurrada → workflow **CI** verde (jobs `test` e `release`).
- [ ] Release no GitHub contém: `Jarvas_x.y.z_x64-setup.exe`, `latest.json` e o `.sig`.

## 1. Instalação e abertura
- [ ] Baixar e rodar o `.exe`. Se o SmartScreen alertar ("editor desconhecido"),
      usar **Mais informações → Executar assim mesmo** (sem code signing — fase 2).
- [ ] O app abre carregando a UI do Jarvas (`jarvas.agendavet.vet.br`), **sem barra
      de endereço nem abas**, título "Jarvas", janela 1280×800.

## 2. Sessão persiste
- [ ] Login email/senha → fechar o app → reabrir → **continua logado** (cookie do
      WebView2 persiste no perfil do app).

## 3. Links externos
- [ ] Clicar um link para outro host (ex.: um link externo na conversa) → abre no
      **navegador do sistema**, não dentro do app.

## 4. Tela offline
- [ ] Desligar a rede → reabrir o app → aparece a tela **"Sem conexão"** local.
- [ ] Religar a rede → botão **"Tentar de novo"** carrega a UI remota.

## 5. Auto-update
- [ ] Com o app instalado na `vX`, publicar uma `vX+1` de teste (bump da `version`
      em `src-tauri/tauri.conf.json` + nova tag) → abrir o app na `vX` → ele
      **baixa e instala** a atualização (verificação por assinatura minisign).

---

### Troubleshooting
- **Updater não atualiza / erro de assinatura:** confirmar que o `pubkey` em
  `tauri.conf.json` corresponde ao par cuja **privada** está no secret
  `TAURI_SIGNING_PRIVATE_KEY`. Se trocar o par, atualizar os dois.
- **Build falha por WebView2:** o `webviewInstallMode: downloadBootstrapper`
  instala o runtime sob demanda; Win10/11 modernos já o trazem.
- **Release vazio:** o build/publish só roda em **tag `v*`** e exige o job `test`
  verde antes.
