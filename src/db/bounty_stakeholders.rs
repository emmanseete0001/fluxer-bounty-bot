use fluxer_neptunium::model::id::{Id, marker::UserMarker};

use crate::db::DbManager;

impl DbManager {
    pub async fn list_bounty_stakeholders(
        &self,
        bounty_id: i64,
    ) -> anyhow::Result<Vec<BountyStakeholder>> {
        let raw = sqlx::query_as!(
            BountyStakeholderSchema,
            "SELECT * FROM bounty_stakeholders
            WHERE bounty_id = $1",
            bounty_id,
        )
        .fetch_all(&self.pool)
        .await?;
        raw.into_iter()
            .map(TryFrom::try_from)
            .collect::<Result<Vec<_>, _>>()
    }
}

pub struct BountyStakeholder {
    pub bounty_id: i64,
    pub user_id: Id<UserMarker>,
    pub amount: u64,
    pub note: Option<String>,
}

impl TryFrom<BountyStakeholderSchema> for BountyStakeholder {
    type Error = anyhow::Error;
    fn try_from(value: BountyStakeholderSchema) -> Result<Self, Self::Error> {
        Ok(Self {
            bounty_id: value.bounty_id,
            user_id: value.user_id.cast_unsigned().into(),
            amount: u64::try_from(value.amount)?,
            note: value.note,
        })
    }
}

struct BountyStakeholderSchema {
    bounty_id: i64,
    user_id: i64,
    amount: i64,
    note: Option<String>,
}
