use anima_core::Result;
use async_trait::async_trait;

use crate::traits::{ConnectorMeta, ConnectorQuery, InboxItem};

pub struct GmailConnector;

impl ConnectorMeta for GmailConnector {
    fn source_name(&self) -> &str {
        "gmail"
    }
}

#[async_trait]
impl ConnectorQuery for GmailConnector {
    async fn fetch_inbox(&self) -> Result<Vec<InboxItem>> {
        // TODO: Google Gmail API で未読メールを取得
        Ok(vec![])
    }

    async fn is_pending(&self, _external_id: &str) -> Result<bool> {
        Ok(true)
    }
}

// Gmail は Query のみ（Phase 1 では送信は対象外）
