use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use datafusion::{
    arrow::datatypes::SchemaRef,
    datasource::{TableProvider, TableType},
    error::Result,
    execution::context::SessionState,
    logical_expr::{Expr, TableProviderFilterPushDown},
    physical_plan::{project_schema, ExecutionPlan},
};
use tskv::engine::EngineRef;

use crate::{predicate::Predicate, schema::TableSchema, tskv_exec::TskvExec};

pub struct ClusterTable {
    engine: EngineRef,
    schema: TableSchema,
}

impl ClusterTable {
    pub(crate) async fn create_physical_plan(
        &self,
        projections: &Option<Vec<usize>>,
        predicate: Arc<Predicate>,
        schema: SchemaRef,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let proj_schema = project_schema(&schema, projections.as_ref()).unwrap();
        Ok(Arc::new(TskvExec::new(
            self.schema.db.clone(),
            self.schema.name.clone(),
            proj_schema,
            predicate,
            self.engine.clone(),
        )))
    }

    pub fn new(engine: EngineRef, schema: TableSchema) -> Self {
        ClusterTable { engine, schema }
    }
}

#[async_trait]
impl TableProvider for ClusterTable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.schema.to_arrow_schema()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        _ctx: &SessionState,
        projection: &Option<Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let filter = Arc::new(
            Predicate::default()
                .set_limit(limit)
                .extract_pushed_down_domains(filters, &self.schema)
                .pushdown_exprs(filters),
        );

        return self
            .create_physical_plan(projection, filter.clone(), self.schema())
            .await;
    }
    fn supports_filter_pushdown(&self, _: &Expr) -> Result<TableProviderFilterPushDown> {
        Ok(TableProviderFilterPushDown::Inexact)
    }
}
