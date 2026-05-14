use std::error::Error;
use std::fmt;

use product::{FieldProductDiagnostic, ProductJobDescriptor, ProductJobId};

use crate::runtime::jobs::diagnostics::runtime_job_failure_diagnostic;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RuntimeJobGeneration(u64);

impl RuntimeJobGeneration {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RuntimeJobKey(u64);

impl RuntimeJobKey {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }

    pub fn from_product_job(product_job: &ProductJobDescriptor) -> Self {
        Self(product_job.job_id.raw())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeJobHandle {
    pub key: RuntimeJobKey,
    pub generation: RuntimeJobGeneration,
    pub submission_sequence: u64,
    pub product_job_id: ProductJobId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeJobExecutionMode {
    Serial,
    WorkerPool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeJobExecutorConfig {
    pub execution_mode: RuntimeJobExecutionMode,
    pub worker_threads: usize,
    pub queue_capacity: usize,
    pub completion_drain_budget: usize,
}

impl RuntimeJobExecutorConfig {
    pub fn serial() -> Self {
        Self {
            execution_mode: RuntimeJobExecutionMode::Serial,
            ..Self::default()
        }
    }

    pub fn worker_pool(worker_threads: usize, queue_capacity: usize) -> Self {
        Self {
            execution_mode: RuntimeJobExecutionMode::WorkerPool,
            worker_threads: worker_threads.max(1),
            queue_capacity: queue_capacity.max(1),
            ..Self::default()
        }
    }

    pub fn normalized(mut self) -> Self {
        self.worker_threads = self.worker_threads.max(1);
        self.queue_capacity = self.queue_capacity.max(1);
        self.completion_drain_budget = self.completion_drain_budget.max(1);
        self
    }
}

impl Default for RuntimeJobExecutorConfig {
    fn default() -> Self {
        Self {
            execution_mode: RuntimeJobExecutionMode::Serial,
            worker_threads: 2,
            queue_capacity: 256,
            completion_drain_budget: 256,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum RuntimeJobStatus {
    Completed,
    Failed,
    Stale,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeJobExecutorDiagnostics {
    pub submitted_count: u64,
    pub completed_count: u64,
    pub failed_count: u64,
    pub stale_count: u64,
    pub rejected_count: u64,
    pub pending_count: u64,
    pub buffered_completion_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeJobError {
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

impl RuntimeJobError {
    pub fn new(diagnostics: impl IntoIterator<Item = FieldProductDiagnostic>) -> Self {
        Self {
            diagnostics: diagnostics.into_iter().collect(),
        }
    }

    pub fn message(message: impl Into<String>) -> Self {
        Self::new([runtime_job_failure_diagnostic(message)])
    }
}

impl fmt::Display for RuntimeJobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = self
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.message.as_str())
            .unwrap_or("runtime product job failed");
        write!(f, "{message}")
    }
}

impl Error for RuntimeJobError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeJobSubmissionError {
    pub key: RuntimeJobKey,
    pub generation: RuntimeJobGeneration,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}

impl RuntimeJobSubmissionError {
    pub fn new(
        key: RuntimeJobKey,
        generation: RuntimeJobGeneration,
        diagnostics: impl IntoIterator<Item = FieldProductDiagnostic>,
    ) -> Self {
        Self {
            key,
            generation,
            diagnostics: diagnostics.into_iter().collect(),
        }
    }
}

impl fmt::Display for RuntimeJobSubmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = self
            .diagnostics
            .first()
            .map(|diagnostic| diagnostic.message.as_str())
            .unwrap_or("runtime product job submission failed");
        write!(f, "{message}")
    }
}

impl Error for RuntimeJobSubmissionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeJobCompletion<T> {
    pub handle: RuntimeJobHandle,
    pub product_job: ProductJobDescriptor,
    pub status: RuntimeJobStatus,
    pub output: Option<T>,
    pub diagnostics: Vec<FieldProductDiagnostic>,
}
