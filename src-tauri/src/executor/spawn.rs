use std::process::Stdio;
use tokio::process::{Child, Command};

use super::context::AgentContext;

pub struct SpawnedAgent {
    pub child: Child,
    pub pid: u32,
}

pub fn spawn_agent(ctx: &AgentContext) -> Result<SpawnedAgent, String> {
    std::fs::create_dir_all(&ctx.working_dir)
        .map_err(|e| format!("Failed to create working directory: {e}"))?;

    let mut cmd = Command::new(&ctx.binary);
    cmd.args(&ctx.args)
        .current_dir(&ctx.working_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    for (key, value) in &ctx.env {
        cmd.env(key, value);
    }

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn agent process '{}': {e}", ctx.binary))?;

    let pid = child
        .id()
        .ok_or_else(|| "Failed to get process ID".to_string())?;

    Ok(SpawnedAgent { child, pid })
}
