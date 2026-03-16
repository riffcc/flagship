use std::{
    env,
    ffi::OsString,
    fs,
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    process::{self, Command, ExitStatus},
    sync::Arc,
};

use anyhow::{Context, Result};
use axum::{
    extract::State,
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use tower::ServiceExt;
use tower_http::{services::ServeFile, trace::TraceLayer};
use tracing::{info, warn};

#[derive(Clone)]
struct AppState {
    static_dir: Arc<PathBuf>,
    runtime_config_js: Arc<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeConfig<'a> {
    api_url: &'a str,
    citadel_peers: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommandKind {
    Serve,
    Dev,
    Build,
    Preview,
    Help,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args_os();
    let _bin = args.next();
    let (command, extra_args) = parse_command(args.collect())?;

    match command {
        CommandKind::Serve => {
            init_tracing();
            serve_command().await
        }
        CommandKind::Dev => run_node_command("dev", extra_args),
        CommandKind::Build => run_node_command("build", extra_args),
        CommandKind::Preview => run_node_command("preview", extra_args),
        CommandKind::Help => {
            print_help();
            Ok(())
        }
    }
}

fn parse_command(args: Vec<OsString>) -> Result<(CommandKind, Vec<OsString>)> {
    let mut iter = args.into_iter();
    let command = match iter.next() {
        None => CommandKind::Serve,
        Some(value) => match value.to_string_lossy().as_ref() {
            "serve" => CommandKind::Serve,
            "dev" => CommandKind::Dev,
            "build" => CommandKind::Build,
            "preview" => CommandKind::Preview,
            "help" | "--help" | "-h" => CommandKind::Help,
            other => anyhow::bail!("unknown subcommand `{other}`"),
        },
    };
    Ok((command, iter.collect()))
}

fn print_help() {
    eprintln!(
        "flagship <command>\n\n\
         Commands:\n\
           serve    Serve the built web app with runtime config injection\n\
           dev      Start the existing frontend dev workflow\n\
           build    Build the existing frontend bundle\n\
           preview  Preview the built frontend bundle\n"
    );
}

fn run_node_command(script: &str, extra_args: Vec<OsString>) -> Result<()> {
    let mut command = Command::new("pnpm");
    command.arg(script);
    command.args(extra_args);
    let status = command
        .status()
        .with_context(|| format!("failed to run `pnpm {script}`"))?;
    exit_with_status(status)
}

fn exit_with_status(status: ExitStatus) -> Result<()> {
    match status.code() {
        Some(code) => process::exit(code),
        None => anyhow::bail!("child process terminated by signal"),
    }
}

async fn serve_command() -> Result<()> {
    let static_dir = resolve_static_dir()?;
    let index_file = static_dir.join("index.html");
    if !index_file.is_file() {
        anyhow::bail!("missing index.html in static dir {}", static_dir.display());
    }

    let app_state = AppState {
        static_dir: Arc::new(static_dir.clone()),
        runtime_config_js: Arc::new(build_runtime_config_js()?),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/config.js", get(runtime_config_js))
        .route("/__flagship/runtime", get(runtime_config_json))
        .fallback(get(spa_fallback))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from((resolve_host(), resolve_port()?));
    info!("flagship serve listening on http://{addr}");
    info!("serving static assets from {}", static_dir.display());
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    Ok(())
}

fn init_tracing() {
    let filter = env::var("RUST_LOG").unwrap_or_else(|_| "flagship_cli=info,tower_http=info".into());
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn resolve_static_dir() -> Result<PathBuf> {
    let explicit = env::var("FLAGSHIP_STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("packages/renderer/dist/web"));
    if explicit.is_dir() {
        return Ok(explicit);
    }

    let fallback = PathBuf::from("/srv/flagship");
    if fallback.is_dir() {
        return Ok(fallback);
    }

    anyhow::bail!(
        "static dir not found: {} (and fallback {} missing)",
        explicit.display(),
        fallback.display()
    );
}

fn resolve_host() -> Ipv4Addr {
    env::var("FLAGSHIP_HOST")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(Ipv4Addr::UNSPECIFIED)
}

fn resolve_port() -> Result<u16> {
    env::var("PORT")
        .or_else(|_| env::var("FLAGSHIP_PORT"))
        .unwrap_or_else(|_| "80".into())
        .parse()
        .context("invalid FLAGSHIP_PORT/PORT value")
}

fn build_runtime_config_js() -> Result<String> {
    let api_url = env_or("API_URL", "/api/v1");
    let citadel_peers = env_or("CITADEL_PEERS", &api_url);
    let config = RuntimeConfig {
        api_url: &api_url,
        citadel_peers: &citadel_peers,
    };
    let json = serde_json::to_string_pretty(&config)?;
    Ok(format!("window.__RUNTIME_CONFIG__ = {json};\n"))
}

fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_string())
}

async fn health() -> &'static str {
    "ok"
}

async fn runtime_config_js(State(state): State<AppState>) -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/javascript; charset=utf-8"),
        )],
        [(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-store, max-age=0"),
        )],
        state.runtime_config_js.as_str().to_owned(),
    )
}

async fn runtime_config_json(State(state): State<AppState>) -> Response {
    match parse_runtime_config(state.runtime_config_js.as_str()) {
        Ok(value) => Json(value).into_response(),
        Err(error) => {
            warn!("failed to parse runtime config: {error:#}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn parse_runtime_config(script: &str) -> Result<serde_json::Value> {
    let trimmed = script.trim();
    let json = trimmed
        .strip_prefix("window.__RUNTIME_CONFIG__ = ")
        .context("missing runtime config prefix")?
        .strip_suffix(';')
        .context("missing runtime config suffix")?;
    Ok(serde_json::from_str(json)?)
}

async fn spa_fallback(State(state): State<AppState>, uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    let requested_path = state.static_dir.join(requested);

    if requested.is_empty() || requested.ends_with('/') {
        return serve_index(&state).await;
    }

    if requested_path.is_file() {
        return serve_file(requested_path).await;
    }

    if looks_like_asset_path(uri.path()) {
        return StatusCode::NOT_FOUND.into_response();
    }

    serve_index(&state).await
}

async fn serve_file(path: PathBuf) -> Response {
    match ServeFile::new(path)
        .oneshot(axum::http::Request::new(axum::body::Body::empty()))
        .await
    {
        Ok(response) => response.into_response(),
        Err(error) => {
            warn!("failed serving file: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn serve_index(state: &AppState) -> Response {
    let index_path = state.static_dir.join("index.html");
    match fs::read_to_string(&index_path) {
        Ok(html) => {
            let body = inject_runtime_config_script(&html);
            (
                [
                    (
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("text/html; charset=utf-8"),
                    ),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static("no-store, max-age=0"),
                    ),
                ],
                body,
            )
                .into_response()
        }
        Err(error) => {
            warn!("failed reading index.html: {error}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn inject_runtime_config_script(html: &str) -> String {
    if html.contains("src=\"/config.js\"") || html.contains("src='/config.js'") {
        return html.to_string();
    }

    if let Some(title_index) = html.find("</title>") {
        let mut output = String::with_capacity(html.len() + 32);
        let insert_at = title_index + "</title>".len();
        output.push_str(&html[..insert_at]);
        output.push_str("\n  <script src=\"/config.js\"></script>");
        output.push_str(&html[insert_at..]);
        return output;
    }

    if let Some(head_index) = html.find("</head>") {
        let mut output = String::with_capacity(html.len() + 32);
        output.push_str(&html[..head_index]);
        output.push_str("  <script src=\"/config.js\"></script>\n");
        output.push_str(&html[head_index..]);
        return output;
    }

    html.to_string()
}

fn looks_like_asset_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext,
                "js"
                    | "css"
                    | "wasm"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "gif"
                    | "ico"
                    | "svg"
                    | "webp"
                    | "json"
                    | "txt"
                    | "xml"
                    | "map"
                    | "webmanifest"
                    | "woff"
                    | "woff2"
                    | "ttf"
            )
        })
        .unwrap_or(false)
}
