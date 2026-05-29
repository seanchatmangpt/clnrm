## Challenge Summary

**Overall risk assessment**: MEDIUM

---

## Challenges

### [Medium] Challenge 1: Concurrency Race Condition in Container Reuse Pool Drop-Release Cycle

* **Assumption challenged**: The assumption that `Arc::try_unwrap` inside `tokio::spawn` in `impl Drop for ContainerHandle` will always succeed to recycle the container because the handle itself is being dropped.
* **Attack scenario**: In a multi-threaded execution context (or under tight CPU constraints), a spawned future in `tokio::spawn` executes concurrently before the main thread completes dropping the `ContainerHandle` instance (and specifically its `container` field). When `Arc::try_unwrap(container_arc)` executes inside the spawned task, it finds the reference count of the `Arc` is still 2 (one in the local task argument, one in `self.container` which hasn't finished dropping yet). 
* **Blast radius**: The `Arc::try_unwrap` fails, printing a warning `Container {} still has multiple references, cannot return to pool`. The container is never returned to the `idle_queue` and is leaked from the pool. Over time, high concurrent test executions will cause pool exhaustion (OOM or no containers available for tests).
* **Mitigation**: Instead of trying to unwrap the `Arc` and store `PooledContainer` directly in the `idle_queue`, store `Arc<PooledContainer>` in the `idle_queue`. This avoids the need to run `Arc::try_unwrap` entirely and makes the drop-release cycle lock-free and race-free. Alternatively, perform the pool return synchronously or ensure that the async block waits or checks again if the count is temporarily elevated.

---

## Stress Test Results

* **Multi-threaded drop test under heavy contention** → Expected behavior: 100% of dropped handles are recycled back to the pool → Actual behavior: Containers fail to recycle and log multiple reference warnings under multi-threaded executor scheduler preemptions → **FAIL**

---

## Unchallenged Areas

* **gVisor (runsc) secure sandbox boundaries** — Reason not challenged: Out of scope for code-only network mode without gVisor installed locally.
* **OpenTelemetry exporter connection failures** — Reason not challenged: Telemetry direct-to-disk fallback handles connection dropouts gracefully; no active external OTLP collector is configured.
