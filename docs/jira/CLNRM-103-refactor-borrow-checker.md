---
id: CLNRM-103
summary: Refactor PortAllocator and Log buffering logic
description: Address mutable/immutable borrow checker violations in `port_allocator.rs` and `logs.rs`. Ensure thread-safe access and correct method invocation on inner objects.
priority: Medium
status: Done
---
