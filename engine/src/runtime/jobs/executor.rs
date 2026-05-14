use std::any::{Any, TypeId};
use std::collections::{BTreeMap, VecDeque};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::thread::{self, JoinHandle};

use crossbeam_channel::{Receiver, Sender, TryRecvError, TrySendError};
use product::{FieldProductDiagnostic, ProductJobDescriptor};

use crate::runtime::jobs::diagnostics::{
    runtime_job_backpressure_diagnostic, runtime_job_disconnected_diagnostic,
    runtime_job_error_for_product, runtime_job_panic_diagnostic, runtime_job_stale_diagnostic,
};
use crate::runtime::jobs::task::RuntimeJob;
use crate::runtime::jobs::types::{
    RuntimeJobCompletion, RuntimeJobError, RuntimeJobExecutionMode, RuntimeJobExecutorConfig,
    RuntimeJobExecutorDiagnostics, RuntimeJobGeneration, RuntimeJobHandle, RuntimeJobKey,
    RuntimeJobStatus, RuntimeJobSubmissionError,
};

type ErasedOutput = Box<dyn Any + Send>;

trait ErasedRuntimeJob: Send {
    fn execute_boxed(self: Box<Self>) -> ErasedRuntimeJobOutcome;
}

struct RuntimeJobEnvelope<J>
where
    J: RuntimeJob,
{
    handle: RuntimeJobHandle,
    product_job: ProductJobDescriptor,
    job: J,
}

impl<J> ErasedRuntimeJob for RuntimeJobEnvelope<J>
where
    J: RuntimeJob,
{
    fn execute_boxed(self: Box<Self>) -> ErasedRuntimeJobOutcome {
        let Self {
            handle,
            product_job,
            job,
        } = *self;
        let output_type_id = TypeId::of::<J::Output>();
        match catch_unwind(AssertUnwindSafe(|| job.execute())) {
            Ok(Ok(output)) => ErasedRuntimeJobOutcome {
                handle,
                product_job,
                status: RuntimeJobStatus::Completed,
                output_type_id,
                output: Some(Box::new(output)),
                diagnostics: Vec::new(),
            },
            Ok(Err(err)) => failed_outcome(handle, product_job, output_type_id, err),
            Err(_) => ErasedRuntimeJobOutcome {
                handle,
                product_job,
                status: RuntimeJobStatus::Failed,
                output_type_id,
                output: None,
                diagnostics: vec![runtime_job_panic_diagnostic()],
            },
        }
    }
}

fn failed_outcome(
    handle: RuntimeJobHandle,
    product_job: ProductJobDescriptor,
    output_type_id: TypeId,
    err: RuntimeJobError,
) -> ErasedRuntimeJobOutcome {
    let diagnostics = err
        .diagnostics
        .into_iter()
        .map(|diagnostic| runtime_job_error_for_product(diagnostic, &product_job))
        .collect();
    ErasedRuntimeJobOutcome {
        handle,
        product_job,
        status: RuntimeJobStatus::Failed,
        output_type_id,
        output: None,
        diagnostics,
    }
}

struct ErasedRuntimeJobOutcome {
    handle: RuntimeJobHandle,
    product_job: ProductJobDescriptor,
    status: RuntimeJobStatus,
    output_type_id: TypeId,
    output: Option<ErasedOutput>,
    diagnostics: Vec<FieldProductDiagnostic>,
}

impl ErasedRuntimeJobOutcome {
    fn mark_stale(&mut self, latest: RuntimeJobGeneration) {
        self.status = RuntimeJobStatus::Stale;
        self.output = None;
        self.diagnostics = vec![runtime_job_stale_diagnostic(self.handle, latest)];
    }

    fn into_typed<T: Send + 'static>(self) -> RuntimeJobCompletion<T> {
        let output = self
            .output
            .and_then(|output| output.downcast::<T>().ok())
            .map(|output| *output);
        RuntimeJobCompletion {
            handle: self.handle,
            product_job: self.product_job,
            status: self.status,
            output,
            diagnostics: self.diagnostics,
        }
    }
}

enum WorkerCommand {
    Run(Box<dyn ErasedRuntimeJob>),
    Shutdown,
}

struct WorkerPool {
    sender: Option<Sender<WorkerCommand>>,
    completions: Receiver<ErasedRuntimeJobOutcome>,
    workers: Vec<JoinHandle<()>>,
}

impl WorkerPool {
    fn new(config: &RuntimeJobExecutorConfig) -> Self {
        let (sender, receiver) = crossbeam_channel::bounded(config.queue_capacity.max(1));
        let (completion_sender, completions) = crossbeam_channel::unbounded();
        let worker_count = config.worker_threads.max(1);
        let mut workers = Vec::with_capacity(worker_count);
        for index in 0..worker_count {
            let receiver: Receiver<WorkerCommand> = receiver.clone();
            let completion_sender = completion_sender.clone();
            let worker = thread::Builder::new()
                .name(format!("runenwerk-product-job-{index}"))
                .spawn(move || {
                    while let Ok(command) = receiver.recv() {
                        match command {
                            WorkerCommand::Run(job) => {
                                let _ = completion_sender.send(job.execute_boxed());
                            }
                            WorkerCommand::Shutdown => break,
                        }
                    }
                })
                .expect("runtime product job worker should spawn");
            workers.push(worker);
        }
        Self {
            sender: Some(sender),
            completions,
            workers,
        }
    }

    fn submit(&self, job: Box<dyn ErasedRuntimeJob>) -> Result<(), TrySendError<WorkerCommand>> {
        let Some(sender) = &self.sender else {
            return Err(TrySendError::Disconnected(WorkerCommand::Run(job)));
        };
        sender.try_send(WorkerCommand::Run(job))
    }

    fn try_recv_completed(&self) -> Result<ErasedRuntimeJobOutcome, TryRecvError> {
        self.completions.try_recv()
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            for _ in 0..self.workers.len() {
                let _ = sender.try_send(WorkerCommand::Shutdown);
            }
        }
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}

enum RuntimeJobBackend {
    Serial,
    WorkerPool(WorkerPool),
}

pub struct RuntimeJobExecutorResource {
    config: RuntimeJobExecutorConfig,
    backend: RuntimeJobBackend,
    latest_generations: BTreeMap<RuntimeJobKey, RuntimeJobGeneration>,
    completed: VecDeque<ErasedRuntimeJobOutcome>,
    next_submission_sequence: u64,
    submitted_count: u64,
    completed_count: u64,
    failed_count: u64,
    stale_count: u64,
    rejected_count: u64,
}

impl Default for RuntimeJobExecutorResource {
    fn default() -> Self {
        Self::with_config(RuntimeJobExecutorConfig::default())
    }
}

impl ecs::Resource for RuntimeJobExecutorResource {}

impl RuntimeJobExecutorResource {
    pub fn with_config(config: RuntimeJobExecutorConfig) -> Self {
        let config = config.normalized();
        let backend = match config.execution_mode {
            RuntimeJobExecutionMode::Serial => RuntimeJobBackend::Serial,
            RuntimeJobExecutionMode::WorkerPool => {
                RuntimeJobBackend::WorkerPool(WorkerPool::new(&config))
            }
        };
        Self {
            config,
            backend,
            latest_generations: BTreeMap::new(),
            completed: VecDeque::new(),
            next_submission_sequence: 0,
            submitted_count: 0,
            completed_count: 0,
            failed_count: 0,
            stale_count: 0,
            rejected_count: 0,
        }
    }

    pub fn config(&self) -> &RuntimeJobExecutorConfig {
        &self.config
    }

    pub fn submit<J>(&mut self, job: J) -> Result<RuntimeJobHandle, RuntimeJobSubmissionError>
    where
        J: RuntimeJob,
    {
        let product_job = job.product_job();
        let key = job.key();
        let generation = job.generation();
        self.next_submission_sequence = self.next_submission_sequence.saturating_add(1);
        let handle = RuntimeJobHandle {
            key,
            generation,
            submission_sequence: self.next_submission_sequence,
            product_job_id: product_job.job_id,
        };
        let envelope = RuntimeJobEnvelope {
            handle,
            product_job,
            job,
        };

        match &self.backend {
            RuntimeJobBackend::Serial => {
                self.latest_generations.insert(key, generation);
                self.submitted_count = self.submitted_count.saturating_add(1);
                self.completed.push_back(Box::new(envelope).execute_boxed());
                Ok(handle)
            }
            RuntimeJobBackend::WorkerPool(pool) => match pool.submit(Box::new(envelope)) {
                Ok(()) => {
                    self.latest_generations.insert(key, generation);
                    self.submitted_count = self.submitted_count.saturating_add(1);
                    Ok(handle)
                }
                Err(TrySendError::Full(_)) => {
                    self.rejected_count = self.rejected_count.saturating_add(1);
                    Err(RuntimeJobSubmissionError::new(
                        key,
                        generation,
                        [runtime_job_backpressure_diagnostic(
                            self.config.queue_capacity,
                        )],
                    ))
                }
                Err(TrySendError::Disconnected(_)) => {
                    self.rejected_count = self.rejected_count.saturating_add(1);
                    Err(RuntimeJobSubmissionError::new(
                        key,
                        generation,
                        [runtime_job_disconnected_diagnostic()],
                    ))
                }
            },
        }
    }

    pub fn drain_completed<T>(&mut self) -> Vec<RuntimeJobCompletion<T>>
    where
        T: Send + 'static,
    {
        self.collect_worker_completions();

        let mut completions = Vec::new();
        let mut remaining = VecDeque::new();
        let output_type_id = TypeId::of::<T>();
        while let Some(mut outcome) = self.completed.pop_front() {
            if outcome.output_type_id != output_type_id {
                remaining.push_back(outcome);
                continue;
            }
            if completions.len() >= self.config.completion_drain_budget {
                remaining.push_back(outcome);
                continue;
            }
            self.apply_staleness(&mut outcome);
            self.record_completion_status(outcome.status);
            completions.push(outcome.into_typed::<T>());
        }
        self.completed = remaining;
        completions
    }

    pub fn diagnostics(&mut self) -> RuntimeJobExecutorDiagnostics {
        self.collect_worker_completions();
        let consumed = self
            .completed_count
            .saturating_add(self.failed_count)
            .saturating_add(self.stale_count)
            .saturating_add(self.rejected_count);
        RuntimeJobExecutorDiagnostics {
            submitted_count: self.submitted_count,
            completed_count: self.completed_count,
            failed_count: self.failed_count,
            stale_count: self.stale_count,
            rejected_count: self.rejected_count,
            pending_count: self.submitted_count.saturating_sub(consumed),
            buffered_completion_count: self.completed.len(),
        }
    }

    fn collect_worker_completions(&mut self) {
        if let RuntimeJobBackend::WorkerPool(pool) = &self.backend {
            while let Ok(outcome) = pool.try_recv_completed() {
                self.completed.push_back(outcome);
            }
        }
    }

    fn apply_staleness(&self, outcome: &mut ErasedRuntimeJobOutcome) {
        let Some(latest) = self.latest_generations.get(&outcome.handle.key).copied() else {
            return;
        };
        if latest > outcome.handle.generation {
            outcome.mark_stale(latest);
        }
    }

    fn record_completion_status(&mut self, status: RuntimeJobStatus) {
        match status {
            RuntimeJobStatus::Completed => {
                self.completed_count = self.completed_count.saturating_add(1);
            }
            RuntimeJobStatus::Failed => {
                self.failed_count = self.failed_count.saturating_add(1);
            }
            RuntimeJobStatus::Stale => {
                self.stale_count = self.stale_count.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    use product::{
        ProductIdentity, ProductJobDescriptor, ProductKind, ProductScaleBand, ProductScope,
    };

    use super::*;
    use crate::runtime::jobs::RuntimeJobResult;

    #[derive(Debug)]
    struct TestJob {
        id: u64,
        generation: u64,
        value: u64,
    }

    impl RuntimeJob for TestJob {
        type Output = u64;

        fn product_job(&self) -> ProductJobDescriptor {
            ProductJobDescriptor::new(
                product::ProductJobId::new(self.id),
                ProductKind::new("runtime_test"),
                "engine.runtime.test",
                ProductIdentity::new(self.id),
                ProductScope::non_spatial("runtime-test"),
                ProductScaleBand::Preview,
            )
        }

        fn generation(&self) -> RuntimeJobGeneration {
            RuntimeJobGeneration::new(self.generation)
        }

        fn execute(self) -> RuntimeJobResult<Self::Output> {
            Ok(self.value)
        }
    }

    #[test]
    fn runtime_job_serial_executor_completes_typed_job_deterministically() {
        let mut executor = RuntimeJobExecutorResource::default();

        executor
            .submit(TestJob {
                id: 1,
                generation: 1,
                value: 42,
            })
            .expect("serial job should submit");

        let completions = executor.drain_completed::<u64>();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].status, RuntimeJobStatus::Completed);
        assert_eq!(completions[0].output, Some(42));
    }

    #[test]
    fn runtime_job_worker_executor_matches_serial_result() {
        let mut executor =
            RuntimeJobExecutorResource::with_config(RuntimeJobExecutorConfig::worker_pool(1, 4));

        executor
            .submit(TestJob {
                id: 2,
                generation: 1,
                value: 7,
            })
            .expect("worker job should submit");

        let completions = wait_for_completions::<u64>(&mut executor, 1);
        assert_eq!(completions[0].status, RuntimeJobStatus::Completed);
        assert_eq!(completions[0].output, Some(7));
    }

    #[test]
    fn runtime_job_newer_generation_marks_older_completion_stale() {
        let mut executor = RuntimeJobExecutorResource::default();

        executor
            .submit(TestJob {
                id: 3,
                generation: 1,
                value: 1,
            })
            .unwrap();
        executor
            .submit(TestJob {
                id: 3,
                generation: 2,
                value: 2,
            })
            .unwrap();

        let completions = executor.drain_completed::<u64>();
        assert_eq!(completions.len(), 2);
        assert_eq!(completions[0].status, RuntimeJobStatus::Stale);
        assert_eq!(completions[0].output, None);
        assert_eq!(completions[1].status, RuntimeJobStatus::Completed);
        assert_eq!(completions[1].output, Some(2));
    }

    #[test]
    fn runtime_job_panic_becomes_failed_completion() {
        struct PanicJob;

        impl RuntimeJob for PanicJob {
            type Output = u64;

            fn product_job(&self) -> ProductJobDescriptor {
                ProductJobDescriptor::new(
                    product::ProductJobId::new(4),
                    ProductKind::new("runtime_test"),
                    "engine.runtime.test",
                    ProductIdentity::new(4),
                    ProductScope::non_spatial("runtime-test"),
                    ProductScaleBand::Preview,
                )
            }

            fn generation(&self) -> RuntimeJobGeneration {
                RuntimeJobGeneration::new(1)
            }

            fn execute(self) -> RuntimeJobResult<Self::Output> {
                panic!("intentional runtime job panic")
            }
        }

        let mut executor = RuntimeJobExecutorResource::default();
        executor.submit(PanicJob).unwrap();

        let completions = executor.drain_completed::<u64>();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].status, RuntimeJobStatus::Failed);
        assert!(completions[0].output.is_none());
        assert!(!completions[0].diagnostics.is_empty());
    }

    #[test]
    fn runtime_job_worker_backpressure_is_reported() {
        struct BlockingJob {
            id: u64,
            started: Arc<AtomicBool>,
            release: Arc<AtomicBool>,
        }

        impl RuntimeJob for BlockingJob {
            type Output = u64;

            fn product_job(&self) -> ProductJobDescriptor {
                ProductJobDescriptor::new(
                    product::ProductJobId::new(self.id),
                    ProductKind::new("runtime_test"),
                    "engine.runtime.test",
                    ProductIdentity::new(self.id),
                    ProductScope::non_spatial("runtime-test"),
                    ProductScaleBand::Preview,
                )
            }

            fn generation(&self) -> RuntimeJobGeneration {
                RuntimeJobGeneration::new(1)
            }

            fn execute(self) -> RuntimeJobResult<Self::Output> {
                self.started.store(true, Ordering::SeqCst);
                while !self.release.load(Ordering::SeqCst) {
                    std::thread::sleep(Duration::from_millis(1));
                }
                Ok(self.id)
            }
        }

        let started = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let mut executor =
            RuntimeJobExecutorResource::with_config(RuntimeJobExecutorConfig::worker_pool(1, 1));
        executor
            .submit(BlockingJob {
                id: 10,
                started: started.clone(),
                release: release.clone(),
            })
            .unwrap();
        wait_until(|| started.load(Ordering::SeqCst));
        executor
            .submit(BlockingJob {
                id: 11,
                started: Arc::new(AtomicBool::new(false)),
                release: release.clone(),
            })
            .unwrap();
        let err = executor
            .submit(BlockingJob {
                id: 12,
                started: Arc::new(AtomicBool::new(false)),
                release: release.clone(),
            })
            .expect_err("bounded worker queue should report backpressure");
        release.store(true, Ordering::SeqCst);

        assert!(!err.diagnostics.is_empty());
    }

    fn wait_for_completions<T: Send + 'static>(
        executor: &mut RuntimeJobExecutorResource,
        expected: usize,
    ) -> Vec<RuntimeJobCompletion<T>> {
        let started = Instant::now();
        loop {
            let completions = executor.drain_completed::<T>();
            if completions.len() >= expected {
                return completions;
            }
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "timed out waiting for runtime job completion"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    fn wait_until(condition: impl Fn() -> bool) {
        let started = Instant::now();
        while !condition() {
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "timed out waiting for worker job to start"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
