use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use marzano_core::{api::MatchResult, compact_api::compact};

use marzano_messenger::{
    emit::{Messager, VisibilityLevels},
    output_mode::OutputMode,
    workflows::StatusManager,
};

pub struct JSONLineMessenger<'a> {
    writer: Arc<Mutex<Box<dyn Write + Send + 'a>>>,
    mode: OutputMode,
    min_level: VisibilityLevels,
    status: StatusManager,
}

impl<'a> JSONLineMessenger<'a> {
    pub fn new<W: Write + Send + 'static>(
        writer: W,
        mode: OutputMode,
        min_level: VisibilityLevels,
    ) -> Self {
        Self {
            writer: Arc::new(Mutex::new(Box::new(writer))),
            mode,
            min_level,
            status: StatusManager::new(),
        }
    }
}

impl<'a> Messager for JSONLineMessenger<'a> {
    fn get_min_level(&self) -> VisibilityLevels {
        self.min_level
    }

    fn get_workflow_status(
        &mut self,
    ) -> anyhow::Result<Option<&marzano_messenger::workflows::PackagedWorkflowOutcome>> {
        self.status.get_workflow_status()
    }

    fn finish_workflow(
        &mut self,
        outcome: &marzano_messenger::workflows::PackagedWorkflowOutcome,
    ) -> anyhow::Result<()> {
        self.status.upsert(outcome);
        Ok(())
    }

    fn raw_emit(&mut self, item: &MatchResult) -> anyhow::Result<()> {
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| anyhow!("JSONLineMessenger lock poisoned"))?;
        match self.mode {
            OutputMode::None => {
                // do nothing
            }
            OutputMode::Standard => {
                serde_json::to_writer(&mut *writer, item)?;
                writer.write_all(b"\n")?;
            }
            OutputMode::Compact => {
                serde_json::to_writer(&mut *writer, &compact(item.clone()))?;
                writer.write_all(b"\n")?;
            }
        }

        Ok(())
    }

    fn emit_log(&mut self, log: &marzano_messenger::SimpleLogMessage) -> anyhow::Result<()> {
        log::debug!("Log received over RPC: {:?}", log);
        Ok(())
    }
}
