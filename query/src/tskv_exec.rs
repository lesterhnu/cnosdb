use std::{any::Any, sync::Arc};

use datafusion::{
    arrow::datatypes::SchemaRef,
    error::Result,
    execution::context::TaskContext,
    physical_expr::PhysicalSortExpr,
    physical_plan::{ExecutionPlan, Partitioning, SendableRecordBatchStream, Statistics},
};

use crate::{predicate::PredicateRef, stream::TableScanStream};
use tskv::engine::EngineRef;

#[derive(Debug, Clone)]
pub struct TskvExec {
    // connection
    // db: CustomDataSource,
    db_name: String,
    table_name: String,
    proj_schema: SchemaRef,
    filter: PredicateRef,
    engine: EngineRef,
}

impl TskvExec {
    pub(crate) fn new(
        db_name: String,
        table_name: String,
        proj_schema: SchemaRef,
        filter: PredicateRef,
        engine: EngineRef,
    ) -> Self {
        Self {
            db_name,
            table_name,
            proj_schema,
            filter,
            engine,
        }
    }
    pub fn filter(&self) -> PredicateRef {
        self.filter.clone()
    }
}

impl ExecutionPlan for TskvExec {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> SchemaRef {
        self.proj_schema.clone()
    }

    fn output_partitioning(&self) -> Partitioning {
        Partitioning::UnknownPartitioning(1)
    }

    fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
        None
    }

    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        vec![]
    }

    fn with_new_children(
        self: Arc<Self>,
        _: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(self)
    }

    fn execute(
        &self,
        _partition: usize,
        context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        let batch_size = context.session_config().batch_size();
        Ok(Box::pin(TableScanStream::new(
            self.db_name.clone(),
            self.table_name.clone(),
            self.schema(),
            self.filter(),
            batch_size,
            self.engine.clone(),
        )))
    }

    fn statistics(&self) -> Statistics {
        todo!()
    }
}
