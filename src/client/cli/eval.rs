use crate::client::cli::parser::{CommandNode, Expr};
use crate::client::core::playback_manager::PlaybackManager;
use crate::client::local_api_proxy::ApiProxy;
use log::debug;

struct EvalContext<'a> {
    playback_manager: PlaybackManager<'a>,
}

async fn eval_simple(ctx: &mut EvalContext<'_>, cmd: &CommandNode) -> Option<String> {
    let args_nf: Vec<String> = cmd // args without flags
        .args
        .iter()
        .filter(|arg| !arg.is_flag())
        .map(|arg| arg.text.clone())
        .collect();
    let has_flag = |flag: &str| cmd.args.iter().any(|a| a.is_flag() && a.text == flag);

    debug!("Evaluating command {} with args {:?}", cmd.name, args_nf);

    match cmd.name.as_str() {
        "play" => ctx.playback_manager.play().await,
        "pause" => ctx.playback_manager.pause().await,
        "next" => {
            let n = args_nf
                .first()
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(1);
            ctx.playback_manager.next(n).await
        }
        "previous" => {
            let n = args_nf
                .first()
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(1);
            ctx.playback_manager.previous(n).await
        }
        "volume" => {
            if args_nf.is_empty() {
                ctx.playback_manager.get_volume().await
            } else {
                let level = args_nf.first().and_then(|s| s.parse::<u8>().ok());
                match level {
                    Some(level) if level <= 100 => ctx.playback_manager.set_volume(level).await,
                    _ => Some("Volume level must be between 0 and 100".to_string()),
                }
            }
        }
        "device" => {
            // shell strips quotes so a multi-word name is multiple args
            if !args_nf.is_empty() {
                ctx.playback_manager.device(&args_nf.join(" ")).await
            } else {
                Some("Device name is required".to_string())
            }
        }
        "devices" => {
            let h = has_flag("-h");
            ctx.playback_manager.devices(h).await
        }
        "now" => {
            let h = has_flag("-h");
            ctx.playback_manager.now(h).await
        }
        "queue" => {
            let h = has_flag("-h");
            if args_nf.is_empty() {
                ctx.playback_manager.queue(h).await
            } else {
                ctx.playback_manager.queue_add(args_nf).await
            }
        }
        "recent" => {
            let h = has_flag("-h");
            let n = args_nf
                .first()
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(20);
            ctx.playback_manager.recent(n, h).await
        }
        _ => Some("Unknown command".to_string()),
    }
}

pub async fn eval(api_proxy: &mut ApiProxy, expr: &Expr) -> Option<String> {
    let mut ctx = EvalContext {
        playback_manager: PlaybackManager::new(api_proxy),
    };
    match expr {
        Expr::Command(cmd) => eval_simple(&mut ctx, cmd).await,
        Expr::Binary(op, _, _) => Some(format!("Error: `{}` is not supported yet", op)),
    }
}
