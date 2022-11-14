use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder, Row, Transaction};
use uuid::Uuid;

use std::collections::HashMap;

use super::entity::*;
use crate::{error::*, primitives::*};

pub struct Entries {
    pool: PgPool,
}

#[derive(Debug)]
pub(crate) struct StagedEntry {
    pub(crate) account_id: AccountId,
    pub(crate) entry_id: EntryId,
    pub(crate) units: Decimal,
    pub(crate) currency: String,
    pub(crate) direction: DebitOrCredit,
}

impl Entries {
    pub fn new(pool: &PgPool) -> Self {
        Self { pool: pool.clone() }
    }

    pub(crate) async fn create_all<'a>(
        &self,
        journal_id: JournalId,
        transaction_id: TransactionId,
        entries: Vec<NewEntry>,
        mut tx: Transaction<'a, Postgres>,
    ) -> Result<(Vec<StagedEntry>, Transaction<'a, Postgres>), SqlxLedgerError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"INSERT INTO entries
                (id, version, transaction_id, journal_id, entry_type, layer,
                 units, currency, direction, description, sequence, account_id)"#,
        );
        let mut partial_ret = HashMap::new();
        let mut sequence = 1;
        query_builder.push_values(
            entries,
            |mut builder,
             NewEntry {
                 account_id,
                 entry_type,
                 layer,
                 units,
                 currency,
                 direction,
                 description,
             }: NewEntry| {
                builder.push("gen_random_uuid()");
                builder.push("1");
                builder.push_bind(Uuid::from(transaction_id));
                builder.push_bind(Uuid::from(journal_id));
                builder.push_bind(entry_type);
                builder.push_bind(layer);
                builder.push_bind(units);
                builder.push_bind(currency.clone());
                builder.push_bind(direction);
                builder.push_bind(description);
                builder.push_bind(sequence as i32);
                builder.push("(SELECT id FROM accounts WHERE id = ");
                builder.push_bind_unseparated(Uuid::from(account_id));
                builder.push_unseparated(")");
                partial_ret.insert(sequence, (account_id, units, currency, direction));
                sequence += 1;
            },
        );
        query_builder.push("RETURNING id");
        let query = query_builder.build();
        let records = query.fetch_all(&mut tx).await?;

        let mut ret = Vec::new();
        sequence = 1;
        for r in records {
            let entry_id = Uuid::from_slice(
                r.try_get_raw("id")
                    .expect("Couldn't get raw id")
                    .as_bytes()
                    .expect("Couldn't convert to bytes"),
            )
            .expect("Couldn't convert to Uuid");
            let (account_id, units, currency, direction) =
                partial_ret.remove(&sequence).expect("sequence not found");
            ret.push(StagedEntry {
                entry_id: entry_id.into(),
                account_id,
                units,
                currency,
                direction,
            });
            sequence += 1;
        }

        Ok((ret, tx))
    }
}