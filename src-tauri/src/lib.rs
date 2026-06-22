//! Jarvas Desktop — thin client.
//!
//! A UI é **remota** (`https://jarvas.agendavet.vet.br`), carregada num WebView2.
//! Toda lógica nativa vive aqui no lado Rust; a página remota roda como webview
//! comum, **sem acesso ao IPC do Tauri** (em Tauri 2 domínios remotos não recebem
//! IPC por padrão e não habilitamos `dangerousRemoteDomainIpcAccess`).

use std::net::ToSocketAddrs;
use std::time::Duration;

use tauri::{Url, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;

/// URL da UI remota carregada na janela.
const REMOTE_URL: &str = "https://jarvas.agendavet.vet.br";
/// Host considerado "interno" (navegação permitida dentro do app).
const REMOTE_HOST: &str = "jarvas.agendavet.vet.br";
/// Owner/repo usados para montar o endpoint do updater (espelha o config).
const REPO_OWNER: &str = "melowesley";
const REPO_NAME: &str = "jarvas-desktop";
/// Tempo máximo da checagem de conectividade no startup.
const REACHABILITY_TIMEOUT: Duration = Duration::from_secs(3);

/// `true` quando a URL pertence ao host do Jarvas (match de host **exato**,
/// não sufixo — `api.jarvas...` e `jarvas...evil.com` são externos).
pub fn is_internal(url: &Url) -> bool {
    url.host_str() == Some(REMOTE_HOST)
}

/// Monta a URL do `latest.json` no GitHub Releases.
///
/// O endpoint canônico do updater fica em `tauri.conf.json`
/// (`plugins.updater.endpoints`); esta função espelha esse formato e é usada
/// para logar/verificar (coberta por teste para travar o formato).
pub fn update_endpoint(owner: &str, repo: &str) -> String {
    format!("https://github.com/{owner}/{repo}/releases/latest/download/latest.json")
}

/// Checa se o servidor remoto está acessível via um connect TCP curto na 443.
/// Falha de DNS (offline) ou timeout → `false`. Não faz request HTTP: só prova
/// que há rede + servidor de pé, o suficiente para decidir online vs. offline.
fn server_reachable(host: &str, port: u16, timeout: Duration) -> bool {
    match (host, port).to_socket_addrs() {
        Ok(addrs) => addrs
            .filter_map(|addr| std::net::TcpStream::connect_timeout(&addr, timeout).ok())
            .next()
            .is_some(),
        Err(_) => false,
    }
}

/// Checagem de atualização assíncrona. Nunca bloqueia nem derruba o app:
/// qualquer falha (sem rede, sem release, assinatura inválida) é logada e ignorada.
async fn check_update(app: tauri::AppHandle) {
    eprintln!(
        "[updater] endpoint: {}",
        update_endpoint(REPO_OWNER, REPO_NAME)
    );
    match app.updater() {
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => {
                eprintln!("[updater] nova versão {} — baixando…", update.version);
                match update
                    .download_and_install(|_chunk, _total| {}, || eprintln!("[updater] instalado"))
                    .await
                {
                    Ok(_) => eprintln!("[updater] atualização aplicada"),
                    Err(e) => eprintln!("[updater] falha ao instalar (ignorado): {e}"),
                }
            }
            Ok(None) => eprintln!("[updater] já está na versão mais recente"),
            Err(e) => eprintln!("[updater] checagem falhou (ignorado): {e}"),
        },
        Err(e) => eprintln!("[updater] indisponível (ignorado): {e}"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // 1. Decide a URL inicial: remota se o servidor responde, senão a tela offline local.
            let initial = if server_reachable(REMOTE_HOST, 443, REACHABILITY_TIMEOUT) {
                WebviewUrl::External(REMOTE_URL.parse().expect("REMOTE_URL inválida"))
            } else {
                eprintln!("[startup] servidor inacessível — abrindo tela offline");
                WebviewUrl::App("offline.html".into())
            };

            // 2. Cria a janela. Links externos abrem no browser do sistema.
            let opener_handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", initial)
                .title("Jarvas")
                .inner_size(1280.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .resizable(true)
                .on_navigation(move |url| {
                    if is_internal(url) {
                        return true; // navegação interna segue
                    }
                    // externa: cancela na webview e abre no browser padrão
                    if let Err(e) = opener_handle.opener().open_url(url.as_str(), None::<&str>) {
                        eprintln!("[nav] falha ao abrir link externo: {e}");
                    }
                    false
                })
                .build()?;

            // 3. Updater em background — não bloqueia a abertura.
            let updater_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                check_update(updater_handle).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Jarvas Desktop");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_do_host_jarvas_e_interna() {
        for raw in [
            "https://jarvas.agendavet.vet.br",
            "https://jarvas.agendavet.vet.br/",
            "https://jarvas.agendavet.vet.br/workspace/email",
            "https://jarvas.agendavet.vet.br:443/chat?x=1#frag",
        ] {
            let u = Url::parse(raw).unwrap();
            assert!(is_internal(&u), "{raw} deveria ser interna");
        }
    }

    #[test]
    fn outros_hosts_sao_externos() {
        for raw in [
            "https://google.com",
            "https://onyx.agendavet.vet.br",
            "https://api.jarvas.agendavet.vet.br",          // subdomínio ≠ host exato
            "https://jarvas.agendavet.vet.br.attacker.com", // sufixo malicioso
            "http://jarvas-agendavet.vet.br",
        ] {
            let u = Url::parse(raw).unwrap();
            assert!(!is_internal(&u), "{raw} deveria ser externa");
        }
    }

    #[test]
    fn endpoint_tem_formato_esperado() {
        assert_eq!(
            update_endpoint("melowesley", "jarvas-desktop"),
            "https://github.com/melowesley/jarvas-desktop/releases/latest/download/latest.json"
        );
    }

    #[test]
    fn endpoint_usa_os_argumentos() {
        assert_eq!(
            update_endpoint("acme", "app"),
            "https://github.com/acme/app/releases/latest/download/latest.json"
        );
    }

    #[test]
    fn endpoint_bate_com_o_host_interno() {
        // Garante que o endpoint do updater aponta para o GitHub e não para o host da UI.
        let ep = update_endpoint(REPO_OWNER, REPO_NAME);
        let u = Url::parse(&ep).unwrap();
        assert_eq!(u.host_str(), Some("github.com"));
        assert!(!is_internal(&u));
    }
}
