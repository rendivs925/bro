use async_trait::async_trait;
use domain::entities::voice_command::VoiceCommand;
use shared::error::Result;
use shared::types::CommandId;

#[async_trait]
pub trait CommandRepository: Send + Sync {
    async fn save(&self, command: &VoiceCommand) -> Result<()>;
    async fn find_by_id(&self, id: &CommandId) -> Result<Option<VoiceCommand>>;
    async fn find_all(&self) -> Result<Vec<VoiceCommand>>;
    async fn delete(&self, id: &CommandId) -> Result<()>;
}
