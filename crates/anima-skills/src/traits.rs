use std::sync::Arc;

use anima_brain::AnthropicClient;
use anima_connectors::{ConnectorMutation, ConnectorQuery};
use anima_context::ContextPipeline;
use anima_core::Result;
use anima_cortex::CortexRepo;
use anima_gateway::Gateway;
use async_trait::async_trait;

/// Shared context available to all skills.
pub struct SkillContext {
    pub cortex: Arc<CortexRepo>,
    pub brain: Arc<AnthropicClient>,
    pub context_engine: Arc<ContextPipeline>,
    pub gateway: Arc<dyn Gateway>,

    /// 読み取り専用コネクタ（観測レイヤー）。
    /// ユーザーが読み取り権限を許可したサービスのみが含まれる。
    pub queries: Vec<Arc<dyn ConnectorQuery>>,

    /// 書き込みコネクタ（干渉レイヤー）。
    /// ユーザーが書き込み権限を許可したサービスのみが含まれる。
    pub mutations: Vec<Arc<dyn ConnectorMutation>>,
}

impl SkillContext {
    /// 指定サービスの Mutation コネクタを取得する。
    pub fn mutation_for(&self, source: &str) -> Option<&dyn ConnectorMutation> {
        self.mutations
            .iter()
            .find(|m| m.source_name() == source)
            .map(|m| m.as_ref())
    }
}

/// Trait for predefined workflow patterns.
#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, ctx: &SkillContext, args: &str) -> Result<()>;
}
