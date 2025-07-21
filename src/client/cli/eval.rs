use crate::client::cli::parser::{Arg, CommandNode, ParseError};
use crate::client::core::playback_manager::PlaybackManager;
use crate::client::local_api_proxy::ApiProxy;

struct EvalContext<'a> {
    playback_manager: PlaybackManager<'a>,
}

async fn eval_simple(
    ctx: &mut EvalContext<'_>,
    command: String,
    args: Vec<String>,
) -> Option<String> {
    let args_nf: Vec<String> = args // args without flags
        .clone()
        .into_iter()
        .filter(|arg| !arg.starts_with('-'))
        .map(|arg| arg.trim().trim_matches('"').to_string())
        .collect();

    println!("Evaluating command: {} with args: {:?}\n", command, args_nf);

    match command.as_str() {
        "play" => ctx.playback_manager.play().await,
        "pause" => ctx.playback_manager.pause().await,
        "next" => {
            let n = args_nf
                .get(0)
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(1);
            ctx.playback_manager.next(n).await
        }
        "previous" => {
            let n = args_nf
                .get(0)
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(1);
            ctx.playback_manager.previous(n).await
        }
        "volume" => {
            if args_nf.is_empty() {
                ctx.playback_manager.get_volume().await
            } else {
                let level = args_nf.get(0).and_then(|s| s.parse::<u8>().ok());
                ctx.playback_manager.set_volume(level.unwrap_or(100)).await
            }
        }
        "device" => {
            if let Some(name) = args_nf.get(0) {
                ctx.playback_manager.device(name).await
            } else {
                Some("Device name is required".to_string())
            }
        }
        "devices" => {
            let h = args.contains(&"-h".to_string());
            ctx.playback_manager.devices(h).await
        }
        "now" => {
            let h = args.contains(&"-h".to_string());
            ctx.playback_manager.now(h).await
        }
        "queue" => {
            let h = args.contains(&"-h".to_string());
            if args_nf.is_empty() {
                ctx.playback_manager.queue(h).await
            } else {
                ctx.playback_manager.queue_add(args_nf).await
            }
        }
        "recent" => {
            let h = args.contains(&"-h".to_string());
            let n = args_nf
                .get(0)
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(20);
            ctx.playback_manager.recent(n, h).await
        }
        _ => Some("Unknown command".to_string()),
    }
}

pub async fn eval(api_proxy: &mut ApiProxy, cmd: &CommandNode) -> Option<String> {
    let mut ctx = EvalContext {
        playback_manager: PlaybackManager::new(api_proxy),
    };
    return eval_rec(&mut ctx, cmd).await;
}

async fn eval_rec(ctx: &mut EvalContext<'_>, cmd: &CommandNode) -> Option<String> {
    let mut evaluated_args: Vec<String> = Vec::new();
    for (i, arg) in cmd.args.iter().enumerate() {
        match arg {
            Arg::Command(subcmd) => {
                // Recursively evaluate subcommands
                evaluated_args.push(Box::pin(eval_rec(ctx, subcmd)).await?);
            }
            Arg::Text(text) => {
                if !text.is_empty() {
                    evaluated_args.push(text.clone());
                }
            }
        }
    }

    return eval_simple(ctx, cmd.name.clone(), evaluated_args).await;
}
