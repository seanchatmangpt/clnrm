use clnrm_core::backend::{ContainerPool, PoolConfig};
use clnrm_core::service::port_allocator::{AllocationStrategy, PortAllocator};
use proptest::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Debug, Clone)]
enum PortAction {
    Allocate {
        service: String,
        preferred: Option<u16>,
    },
    Release {
        service: String,
        port: u16,
    },
    ReleaseAll {
        service: String,
    },
}

fn port_action_strategy() -> impl Strategy<Value = PortAction> {
    let service_names = prop::sample::select(vec![
        "svc_a".to_string(),
        "svc_b".to_string(),
        "svc_c".to_string(),
    ]);
    let optional_port = prop::option::of(10000u16..10050u16);

    prop_oneof![
        (service_names.clone(), optional_port)
            .prop_map(|(service, preferred)| PortAction::Allocate { service, preferred }),
        (service_names.clone(), 10000u16..10050u16)
            .prop_map(|(service, port)| PortAction::Release { service, port }),
        service_names
            .clone()
            .prop_map(|service| PortAction::ReleaseAll { service }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_port_allocator_consistency(actions in prop::collection::vec(port_action_strategy(), 1..50)) {
        let mut allocator = PortAllocator::new(AllocationStrategy::Sequential { next: 10000 })
            .with_range(10000..10050);

        let mut expected_allocations: HashMap<String, HashSet<u16>> = HashMap::new();
        let mut all_allocated: HashSet<u16> = HashSet::new();

        for action in actions {
            match action {
                PortAction::Allocate { service, preferred } => {
                    if let Ok(port) = allocator.allocate(&service, preferred) {
                        expected_allocations.entry(service.clone()).or_default().insert(port);
                        all_allocated.insert(port);
                    }
                }
                PortAction::Release { service, port } => {
                    allocator.release(&service, port);
                    if let Some(ports) = expected_allocations.get_mut(&service) {
                        if ports.remove(&port) {
                            let mut still_used = false;
                            for p in expected_allocations.values() {
                                if p.contains(&port) {
                                    still_used = true;
                                }
                            }
                            if !still_used {
                                all_allocated.remove(&port);
                            }
                        }
                    }
                }
                PortAction::ReleaseAll { service } => {
                    allocator.release_all(&service);
                    if let Some(ports) = expected_allocations.remove(&service) {
                        for port in ports {
                            let mut still_used = false;
                            for p in expected_allocations.values() {
                                if p.contains(&port) {
                                    still_used = true;
                                }
                            }
                            if !still_used {
                                all_allocated.remove(&port);
                            }
                        }
                    }
                }
            }

            // Invariant: allocator.get_allocated(service) matches expected_allocations
            for (service, ports) in &expected_allocations {
                let actual = allocator.get_allocated(service);
                let actual_set: HashSet<u16> = actual.into_iter().collect();
                prop_assert_eq!(&actual_set, ports);
            }
        }

        // Ensure no leaks when we release everything
        for service in expected_allocations.keys().cloned().collect::<Vec<_>>() {
            allocator.release_all(&service);
        }

        for service in expected_allocations.keys() {
            let actual = allocator.get_allocated(service);
            prop_assert!(actual.is_empty());
        }
    }
}

#[derive(Debug, Clone)]
enum PoolAction {
    Acquire,
    Release(usize),
}

fn pool_action_strategy() -> impl Strategy<Value = PoolAction> {
    prop_oneof![
        Just(PoolAction::Acquire),
        (0..10usize).prop_map(PoolAction::Release),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))] // Fewer cases since async/pool overhead

    #[test]
    fn test_container_pool_consistency(actions in prop::collection::vec(pool_action_strategy(), 1..20)) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let config = PoolConfig {
                max_size: 5,
                min_idle: 0,
                ..Default::default()
            };

            let pool = match ContainerPool::new(config).await {
                Ok(p) => p,
                Err(_) => return, // Skip if backend init fails
            };

            let mut acquired_handles = Vec::new();

            for action in actions {
                match action {
                    PoolAction::Acquire => {
                        if let Ok(container) = pool.acquire().await {
                            acquired_handles.push(container);
                        }
                    }
                    PoolAction::Release(idx) => {
                        if !acquired_handles.is_empty() {
                            let i = idx % acquired_handles.len();
                            let container = acquired_handles.remove(i);
                            let _ = pool.release(container).await;
                        }
                    }
                }

                let stats = pool.stats();
                assert!(stats.active <= 5);
                assert_eq!(stats.active as usize, acquired_handles.len());
            }

            // Release all
            while let Some(container) = acquired_handles.pop() {
                let _ = pool.release(container).await;
            }

            let stats = pool.stats();
            assert_eq!(stats.active, 0);

            let _ = pool.shutdown().await;
        });
    }
}
