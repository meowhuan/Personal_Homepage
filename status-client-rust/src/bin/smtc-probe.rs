#[cfg(not(windows))]
fn main() {
    println!("smtc-probe only supports Windows.");
}

#[cfg(windows)]
fn main() {
    use pollster::block_on;
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    let result = block_on(async {
        let manager_op = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?;
        let manager = manager_op.await?;

        let mut chosen: Option<GlobalSystemMediaTransportControlsSession> = None;
        if let Ok(sessions) = manager.GetSessions() {
            let len = sessions.Size().unwrap_or(0);
            for idx in 0..len {
                let Ok(session) = sessions.GetAt(idx) else {
                    continue;
                };
                let Ok(playback) = session.GetPlaybackInfo() else {
                    continue;
                };
                let Ok(status) = playback.PlaybackStatus() else {
                    continue;
                };
                if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                    chosen = Some(session);
                    break;
                }
            }
        }

        let session = chosen.or_else(|| manager.GetCurrentSession().ok());
        let Some(session) = session else {
            return Ok::<String, windows::core::Error>(
                "{\"ok\":false,\"reason\":\"no_session\"}".to_string(),
            );
        };

        let playing = session
            .GetPlaybackInfo()
            .and_then(|p| p.PlaybackStatus())
            .map(|s| s == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
            .unwrap_or(false);

        let media_op = session.TryGetMediaPropertiesAsync()?;
        let media = media_op.await?;

        let title = media.Title().ok().map(|s| s.to_string()).unwrap_or_default();
        let artist = media.Artist().ok().map(|s| s.to_string()).unwrap_or_default();
        let source = session
            .SourceAppUserModelId()
            .ok()
            .map(|s| s.to_string())
            .unwrap_or_default();

        Ok(format!(
            "{{\"ok\":true,\"playing\":{},\"title\":\"{}\",\"artist\":\"{}\",\"source\":\"{}\"}}",
            if playing { "true" } else { "false" },
            escape(&title),
            escape(&artist),
            escape(&source)
        ))
    });

    match result {
        Ok(s) => println!("{s}"),
        Err(e) => println!(
            "{{\"ok\":false,\"reason\":\"request_failed\",\"error\":\"{}\"}}",
            escape(&e.to_string())
        ),
    }
}

#[cfg(windows)]
fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
