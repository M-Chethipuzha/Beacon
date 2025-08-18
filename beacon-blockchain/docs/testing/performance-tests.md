# Performance Testing Documentation

## Overview

Performance testing for the BEACON platform ensures the system can handle expected loads, maintains acceptable response times, and scales effectively. This document outlines comprehensive performance testing strategies, tools, and benchmarks.

## Performance Test Categories

### 1. API Performance Tests

#### Response Time Benchmarks

```rust
// benches/api_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use beacon_api::app::create_app;
use beacon_storage::SQLiteStorage;
use axum::http::Request;
use tower::ServiceExt;
use std::time::Duration;

async fn setup_test_app() -> axum::Router {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();
    create_app(storage).await
}

fn benchmark_authentication(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(setup_test_app());

    c.bench_function("auth_login", |b| {
        b.to_async(&rt).iter(|| async {
            let request = Request::builder()
                .method("POST")
                .uri("/auth/login")
                .header("content-type", "application/json")
                .body(r#"{"username": "admin", "password": "admin123"}"#.to_string())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            black_box(response);
        });
    });
}

fn benchmark_blockchain_info(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(setup_test_app());

    // Get auth token first
    let token = rt.block_on(async {
        let login_request = Request::builder()
            .method("POST")
            .uri("/auth/login")
            .header("content-type", "application/json")
            .body(r#"{"username": "admin", "password": "admin123"}"#.to_string())
            .unwrap();

        let response = app.clone().oneshot(login_request).await.unwrap();
        // Extract token from response
        "test_token".to_string() // Simplified for benchmark
    });

    c.bench_function("blockchain_info", |b| {
        b.to_async(&rt).iter(|| async {
            let request = Request::builder()
                .method("GET")
                .uri("/api/v1/blockchain/info")
                .header("authorization", format!("Bearer {}", token))
                .body("".to_string())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            black_box(response);
        });
    });
}

fn benchmark_transaction_submission(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let app = rt.block_on(setup_test_app());
    let token = "test_token"; // Simplified

    let mut group = c.benchmark_group("transaction_submission");

    for tx_size in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::new("args_count", tx_size), tx_size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let args: Vec<String> = (0..size).map(|i| format!("arg_{}", i)).collect();

                let tx_data = serde_json::json!({
                    "chaincode_id": "test-contract",
                    "function": "test_function",
                    "args": args
                });

                let request = Request::builder()
                    .method("POST")
                    .uri("/api/v1/transactions")
                    .header("authorization", format!("Bearer {}", token))
                    .header("content-type", "application/json")
                    .body(tx_data.to_string())
                    .unwrap();

                let response = app.clone().oneshot(request).await.unwrap();
                black_box(response);
            });
        });
    }

    group.finish();
}

criterion_group!(
    api_benches,
    benchmark_authentication,
    benchmark_blockchain_info,
    benchmark_transaction_submission
);
criterion_main!(api_benches);
```

#### Load Testing with Multiple Clients

```rust
// tests/performance/load_tests.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use beacon_api::app::create_app;
use beacon_storage::SQLiteStorage;

#[tokio::test]
async fn test_concurrent_api_load() {
    let storage = SQLiteStorage::new(":memory:").await.unwrap();
    let app = Arc::new(create_app(storage).await);

    struct LoadTestConfig {
        concurrent_users: usize,
        requests_per_user: usize,
        duration: Duration,
    }

    let configs = vec![
        LoadTestConfig { concurrent_users: 10, requests_per_user: 100, duration: Duration::from_secs(30) },
        LoadTestConfig { concurrent_users: 50, requests_per_user: 50, duration: Duration::from_secs(30) },
        LoadTestConfig { concurrent_users: 100, requests_per_user: 25, duration: Duration::from_secs(30) },
    ];

    for config in configs {
        println!("Testing with {} concurrent users, {} requests each",
                config.concurrent_users, config.requests_per_user);

        let start_time = Instant::now();
        let mut join_set = JoinSet::new();

        for user_id in 0..config.concurrent_users {
            let app_clone = app.clone();
            let requests_count = config.requests_per_user;

            join_set.spawn(async move {
                let mut response_times = Vec::new();
                let mut successful_requests = 0;
                let mut failed_requests = 0;

                // Login
                let token = login_user(&app_clone, &format!("user_{}", user_id)).await
                    .unwrap_or_else(|_| "fallback_token".to_string());

                for _ in 0..requests_count {
                    let request_start = Instant::now();

                    match make_api_request(&app_clone, &token).await {
                        Ok(_) => {
                            successful_requests += 1;
                            response_times.push(request_start.elapsed());
                        }
                        Err(_) => failed_requests += 1,
                    }
                }

                (user_id, successful_requests, failed_requests, response_times)
            });
        }

        // Collect results
        let mut total_successful = 0;
        let mut total_failed = 0;
        let mut all_response_times = Vec::new();

        while let Some(result) = join_set.join_next().await {
            if let Ok((_, successful, failed, response_times)) = result {
                total_successful += successful;
                total_failed += failed;
                all_response_times.extend(response_times);
            }
        }

        let total_duration = start_time.elapsed();
        let total_requests = total_successful + total_failed;
        let requests_per_second = total_requests as f64 / total_duration.as_secs_f64();

        // Calculate response time statistics
        all_response_times.sort();
        let avg_response_time = all_response_times.iter().sum::<Duration>() / all_response_times.len() as u32;
        let p95_response_time = all_response_times[(all_response_times.len() as f64 * 0.95) as usize];
        let p99_response_time = all_response_times[(all_response_times.len() as f64 * 0.99) as usize];

        println!("Results:");
        println!("  Total requests: {}", total_requests);
        println!("  Successful: {}", total_successful);
        println!("  Failed: {}", total_failed);
        println!("  Success rate: {:.2}%", (total_successful as f64 / total_requests as f64) * 100.0);
        println!("  Requests/second: {:.2}", requests_per_second);
        println!("  Average response time: {:?}", avg_response_time);
        println!("  95th percentile: {:?}", p95_response_time);
        println!("  99th percentile: {:?}", p99_response_time);
        println!("---");

        // Assertions for minimum performance requirements
        assert!(requests_per_second > 100.0, "Throughput below minimum requirement");
        assert!(avg_response_time < Duration::from_millis(100), "Average response time too high");
        assert!(p95_response_time < Duration::from_millis(500), "95th percentile too high");
        assert!((total_successful as f64 / total_requests as f64) > 0.99, "Success rate too low");
    }
}

async fn login_user(app: &axum::Router, username: &str) -> Result<String, Box<dyn std::error::Error>> {
    let login_request = axum::http::Request::builder()
        .method("POST")
        .uri("/auth/login")
        .header("content-type", "application/json")
        .body(format!(r#"{{"username": "{}", "password": "password123"}}"#, username))
        .unwrap();

    let response = app.clone().oneshot(login_request).await?;
    // Extract token from response body
    Ok("extracted_token".to_string()) // Simplified
}

async fn make_api_request(app: &axum::Router, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let request = axum::http::Request::builder()
        .method("GET")
        .uri("/api/v1/blockchain/info")
        .header("authorization", format!("Bearer {}", token))
        .body("".to_string())
        .unwrap();

    let response = app.clone().oneshot(request).await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err("Request failed".into())
    }
}
```

### 2. Storage Performance Tests

#### Database Operation Benchmarks

```rust
// benches/storage_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use beacon_storage::{StateStorage, SQLiteStorage, InMemoryStorage};
use std::sync::Arc;

fn benchmark_storage_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("storage_operations");

    // Test different storage backends
    let backends = vec![
        ("sqlite_memory", rt.block_on(SQLiteStorage::new(":memory:")).unwrap()),
        ("sqlite_file", rt.block_on(SQLiteStorage::new("test_perf.db")).unwrap()),
    ];

    for (backend_name, storage) in backends {
        let storage = Arc::new(storage);

        // Benchmark single write operations
        group.bench_with_input(
            BenchmarkId::new("single_write", backend_name),
            &storage,
            |b, storage| {
                b.to_async(&rt).iter(|| async {
                    storage.put_state("test", "key1", "value1", 1).await.unwrap();
                });
            },
        );

        // Benchmark single read operations
        group.bench_with_input(
            BenchmarkId::new("single_read", backend_name),
            &storage,
            |b, storage| {
                // Setup data first
                rt.block_on(async {
                    storage.put_state("test", "read_key", "read_value", 1).await.unwrap();
                });

                b.to_async(&rt).iter(|| async {
                    storage.get_state("test", "read_key").await.unwrap();
                });
            },
        );

        // Benchmark batch operations
        for batch_size in [10, 100, 1000].iter() {
            group.bench_with_input(
                BenchmarkId::new("batch_write", format!("{}_{}", backend_name, batch_size)),
                &(*batch_size, storage.clone()),
                |b, (size, storage)| {
                    b.to_async(&rt).iter(|| async {
                        for i in 0..*size {
                            storage.put_state("test", &format!("batch_key_{}", i), &format!("batch_value_{}", i), 1).await.unwrap();
                        }
                    });
                },
            );
        }

        // Benchmark range queries
        for range_size in [10, 100, 1000].iter() {
            group.bench_with_input(
                BenchmarkId::new("range_query", format!("{}_{}", backend_name, range_size)),
                &(*range_size, storage.clone()),
                |b, (size, storage)| {
                    // Setup range data
                    rt.block_on(async {
                        for i in 0..*size {
                            storage.put_state("test", &format!("range_key_{:04}", i), &format!("value_{}", i), 1).await.unwrap();
                        }
                    });

                    b.to_async(&rt).iter(|| async {
                        storage.get_state_range("test", "range_key_0000", "range_key_9999", *size).await.unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

fn benchmark_transaction_storage(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = Arc::new(rt.block_on(SQLiteStorage::new(":memory:")).unwrap());

    c.bench_function("transaction_store", |b| {
        b.to_async(&rt).iter(|| async {
            let tx = beacon_core::types::Transaction {
                id: format!("tx_{}", uuid::Uuid::new_v4()),
                chaincode_id: "test-contract".to_string(),
                function: "test_function".to_string(),
                args: vec!["arg1".to_string(), "arg2".to_string()],
                timestamp: chrono::Utc::now(),
                signature: "test_signature".to_string(),
            };

            storage.store_transaction(&tx).await.unwrap();
        });
    });

    c.bench_function("transaction_retrieve", |b| {
        // Setup test transaction
        let test_tx_id = rt.block_on(async {
            let tx = beacon_core::types::Transaction {
                id: "perf_test_tx".to_string(),
                chaincode_id: "test-contract".to_string(),
                function: "test_function".to_string(),
                args: vec![],
                timestamp: chrono::Utc::now(),
                signature: "signature".to_string(),
            };
            storage.store_transaction(&tx).await.unwrap();
            tx.id
        });

        b.to_async(&rt).iter(|| async {
            storage.get_transaction(&test_tx_id).await.unwrap();
        });
    });
}

criterion_group!(
    storage_benches,
    benchmark_storage_operations,
    benchmark_transaction_storage
);
criterion_main!(storage_benches);
```

#### Memory Usage Analysis

```rust
// tests/performance/memory_tests.rs
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct TracingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TracingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: TracingAllocator = TracingAllocator;

#[tokio::test]
async fn test_memory_usage_under_load() {
    let initial_memory = ALLOCATED.load(Ordering::SeqCst);

    // Create storage and API
    let storage = beacon_storage::SQLiteStorage::new(":memory:").await.unwrap();
    let app = beacon_api::app::create_app(storage).await;

    let after_setup_memory = ALLOCATED.load(Ordering::SeqCst);
    println!("Memory after setup: {} bytes", after_setup_memory - initial_memory);

    // Simulate load
    for i in 0..1000 {
        let request = axum::http::Request::builder()
            .method("GET")
            .uri("/health")
            .body("".to_string())
            .unwrap();

        let _ = tower::ServiceExt::oneshot(app.clone(), request).await.unwrap();

        if i % 100 == 0 {
            let current_memory = ALLOCATED.load(Ordering::SeqCst);
            println!("Memory after {} requests: {} bytes", i, current_memory - initial_memory);
        }
    }

    let final_memory = ALLOCATED.load(Ordering::SeqCst);
    let memory_growth = final_memory - after_setup_memory;

    println!("Total memory growth: {} bytes", memory_growth);

    // Assert memory growth is reasonable (less than 10MB for 1000 requests)
    assert!(memory_growth < 10 * 1024 * 1024, "Memory growth too high: {} bytes", memory_growth);
}
```

### 3. Chaincode Performance Tests

#### Execution Benchmarks

```rust
// benches/chaincode_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use beacon_chaincode::{ChainCodeExecutor, ExecutionContext};
use beacon_storage::SQLiteStorage;
use std::sync::Arc;

fn benchmark_chaincode_execution(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = Arc::new(rt.block_on(SQLiteStorage::new(":memory:")).unwrap());
    let executor = ChainCodeExecutor::new();

    let mut group = c.benchmark_group("chaincode_execution");

    // Benchmark different chaincode functions
    let test_cases = vec![
        ("simple_set", "setValue", vec!["key1".to_string(), "value1".to_string()]),
        ("simple_get", "getValue", vec!["key1".to_string()]),
        ("complex_calculation", "calculateTotal", vec!["100".to_string(), "200".to_string(), "300".to_string()]),
    ];

    for (test_name, function, args) in test_cases {
        group.bench_with_input(
            BenchmarkId::new("execution", test_name),
            &(function, args),
            |b, (func, args)| {
                b.to_async(&rt).iter(|| async {
                    let mut ctx = ExecutionContext::new(
                        "test-contract".to_string(),
                        storage.clone(),
                        1,
                    );

                    executor.execute(&mut ctx, func, args.clone()).await.unwrap();
                });
            },
        );
    }

    // Benchmark with different argument sizes
    for arg_count in [1, 10, 100].iter() {
        let args: Vec<String> = (0..*arg_count).map(|i| format!("arg_{}", i)).collect();

        group.bench_with_input(
            BenchmarkId::new("args_scaling", arg_count),
            &args,
            |b, args| {
                b.to_async(&rt).iter(|| async {
                    let mut ctx = ExecutionContext::new(
                        "test-contract".to_string(),
                        storage.clone(),
                        1,
                    );

                    executor.execute(&mut ctx, "processArgs", args.clone()).await.unwrap();
                });
            },
        );
    }

    group.finish();
}

fn benchmark_state_operations_in_chaincode(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = Arc::new(rt.block_on(SQLiteStorage::new(":memory:")).unwrap());
    let executor = ChainCodeExecutor::new();

    c.bench_function("chaincode_state_write", |b| {
        b.to_async(&rt).iter(|| async {
            let mut ctx = ExecutionContext::new(
                "test-contract".to_string(),
                storage.clone(),
                1,
            );

            // Simulate chaincode that writes multiple state values
            for i in 0..10 {
                ctx.put_state(&format!("key_{}", i), &format!("value_{}", i)).await.unwrap();
            }
        });
    });

    c.bench_function("chaincode_state_read", |b| {
        // Setup state first
        rt.block_on(async {
            let mut ctx = ExecutionContext::new(
                "test-contract".to_string(),
                storage.clone(),
                1,
            );

            for i in 0..10 {
                ctx.put_state(&format!("read_key_{}", i), &format!("read_value_{}", i)).await.unwrap();
            }
        });

        b.to_async(&rt).iter(|| async {
            let ctx = ExecutionContext::new(
                "test-contract".to_string(),
                storage.clone(),
                1,
            );

            // Read multiple state values
            for i in 0..10 {
                ctx.get_state(&format!("read_key_{}", i)).await.unwrap();
            }
        });
    });
}

criterion_group!(
    chaincode_benches,
    benchmark_chaincode_execution,
    benchmark_state_operations_in_chaincode
);
criterion_main!(chaincode_benches);
```

### 4. Network Performance Tests

#### Throughput Testing

```bash
#!/bin/bash
# Network throughput testing script

API_BASE="http://localhost:3000"

echo "=== BEACON Network Performance Tests ==="

# Function to get auth token
get_token() {
    curl -s -X POST "$API_BASE/auth/login" \
        -H "Content-Type: application/json" \
        -d '{"username": "admin", "password": "admin123"}' | \
        jq -r '.access_token'
}

# Test 1: Baseline latency
echo "1. Testing baseline latency..."
TOKEN=$(get_token)

echo "   Measuring response times for health endpoint:"
for i in {1..10}; do
    TIME=$(curl -w "%{time_total}" -s -o /dev/null "$API_BASE/health")
    echo "   Request $i: ${TIME}s"
done

echo "   Measuring response times for authenticated endpoint:"
for i in {1..10}; do
    TIME=$(curl -w "%{time_total}" -s -o /dev/null \
        -H "Authorization: Bearer $TOKEN" \
        "$API_BASE/api/v1/blockchain/info")
    echo "   Request $i: ${TIME}s"
done

# Test 2: Sustained throughput
echo "2. Testing sustained throughput..."

CONCURRENT_USERS=50
REQUESTS_PER_USER=20
TOTAL_REQUESTS=$((CONCURRENT_USERS * REQUESTS_PER_USER))

echo "   Running $TOTAL_REQUESTS requests with $CONCURRENT_USERS concurrent users..."

START_TIME=$(date +%s)

# Use GNU parallel or xargs for concurrent requests
seq 1 $TOTAL_REQUESTS | xargs -n1 -P$CONCURRENT_USERS -I{} bash -c "
    TOKEN=\$(curl -s -X POST \"$API_BASE/auth/login\" \
        -H \"Content-Type: application/json\" \
        -d '{\"username\": \"admin\", \"password\": \"admin123\"}' | \
        jq -r '.access_token')

    curl -s -H \"Authorization: Bearer \$TOKEN\" \
        \"$API_BASE/api/v1/blockchain/info\" > /dev/null
"

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
THROUGHPUT=$((TOTAL_REQUESTS / DURATION))

echo "   Completed $TOTAL_REQUESTS requests in ${DURATION}s"
echo "   Throughput: $THROUGHPUT requests/second"

# Test 3: Large payload handling
echo "3. Testing large payload handling..."

# Create large transaction payload
LARGE_PAYLOAD=$(python3 -c "
import json
args = ['large_value_' + str(i) for i in range(1000)]
payload = {
    'chaincode_id': 'test-contract',
    'function': 'processLargeData',
    'args': args
}
print(json.dumps(payload))
")

echo "   Testing large transaction submission..."
LARGE_TIME=$(curl -w "%{time_total}" -s -o /dev/null \
    -X POST "$API_BASE/api/v1/transactions" \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "$LARGE_PAYLOAD")

echo "   Large payload response time: ${LARGE_TIME}s"

# Test 4: Rate limiting verification
echo "4. Testing rate limiting behavior..."

echo "   Sending requests until rate limited..."
RATE_LIMITED=false
REQUEST_COUNT=0

while [ "$RATE_LIMITED" = false ] && [ $REQUEST_COUNT -lt 150 ]; do
    RESPONSE=$(curl -s -w "%{http_code}" \
        -H "Authorization: Bearer $TOKEN" \
        "$API_BASE/api/v1/blockchain/info")

    REQUEST_COUNT=$((REQUEST_COUNT + 1))

    if [[ "$RESPONSE" == *"429"* ]]; then
        echo "   Rate limiting triggered at request $REQUEST_COUNT"
        RATE_LIMITED=true
    fi
done

if [ "$RATE_LIMITED" = false ]; then
    echo "   Rate limiting not triggered within $REQUEST_COUNT requests"
fi

echo "=== Performance test completed ==="
```

#### WebSocket Performance (if applicable)

```rust
// tests/performance/websocket_tests.rs
#[cfg(feature = "websocket")]
mod websocket_tests {
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use std::time::{Duration, Instant};

    #[tokio::test]
    async fn test_websocket_message_throughput() {
        let url = "ws://localhost:3000/ws";

        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (mut write, mut read) = ws_stream.split();

        let message_count = 1000;
        let start_time = Instant::now();

        // Send messages
        for i in 0..message_count {
            let message = Message::Text(format!("test_message_{}", i));
            write.send(message).await.unwrap();
        }

        // Receive responses
        let mut received_count = 0;
        while received_count < message_count {
            if let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(_)) => received_count += 1,
                    _ => continue,
                }
            }
        }

        let duration = start_time.elapsed();
        let messages_per_second = message_count as f64 / duration.as_secs_f64();

        println!("WebSocket throughput: {:.2} messages/second", messages_per_second);
        assert!(messages_per_second > 500.0, "WebSocket throughput too low");
    }

    #[tokio::test]
    async fn test_websocket_concurrent_connections() {
        let concurrent_connections = 50;
        let messages_per_connection = 10;

        let mut join_set = tokio::task::JoinSet::new();

        for conn_id in 0..concurrent_connections {
            join_set.spawn(async move {
                let url = "ws://localhost:3000/ws";
                let (ws_stream, _) = connect_async(url).await.unwrap();
                let (mut write, mut read) = ws_stream.split();

                // Send messages
                for i in 0..messages_per_connection {
                    let message = Message::Text(format!("conn_{}_msg_{}", conn_id, i));
                    write.send(message).await.unwrap();
                }

                // Read responses
                for _ in 0..messages_per_connection {
                    read.next().await;
                }

                conn_id
            });
        }

        let start_time = Instant::now();
        let mut completed_connections = 0;

        while let Some(result) = join_set.join_next().await {
            if result.is_ok() {
                completed_connections += 1;
            }
        }

        let duration = start_time.elapsed();

        assert_eq!(completed_connections, concurrent_connections);
        assert!(duration < Duration::from_secs(30), "Concurrent connections took too long");

        println!("Completed {} concurrent WebSocket connections in {:?}",
                completed_connections, duration);
    }
}
```

### 5. Resource Utilization Tests

#### CPU and Memory Monitoring

```rust
// tests/performance/resource_monitoring.rs
use std::process::Command;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, ProcessExt};

#[tokio::test]
async fn test_resource_utilization_under_load() {
    let mut system = System::new_all();

    // Get initial system state
    system.refresh_all();
    let initial_memory = get_process_memory(&system, "beacon-api");
    let initial_cpu = get_process_cpu(&system, "beacon-api");

    println!("Initial state - Memory: {} MB, CPU: {:.2}%",
             initial_memory / 1024 / 1024, initial_cpu);

    // Start load test
    let load_test_handle = tokio::spawn(async {
        run_load_test().await;
    });

    // Monitor resources during load test
    let monitoring_handle = tokio::spawn(async move {
        let mut max_memory = 0;
        let mut max_cpu = 0.0;
        let mut samples = Vec::new();

        for i in 0..60 { // Monitor for 60 seconds
            tokio::time::sleep(Duration::from_secs(1)).await;

            system.refresh_all();
            let current_memory = get_process_memory(&system, "beacon-api");
            let current_cpu = get_process_cpu(&system, "beacon-api");

            max_memory = max_memory.max(current_memory);
            max_cpu = max_cpu.max(current_cpu);

            samples.push((current_memory, current_cpu));

            if i % 10 == 0 {
                println!("Sample {}: Memory: {} MB, CPU: {:.2}%",
                        i, current_memory / 1024 / 1024, current_cpu);
            }
        }

        (max_memory, max_cpu, samples)
    });

    // Wait for both tasks
    let _ = load_test_handle.await;
    let (max_memory, max_cpu, samples) = monitoring_handle.await.unwrap();

    println!("Resource utilization results:");
    println!("  Max memory: {} MB", max_memory / 1024 / 1024);
    println!("  Max CPU: {:.2}%", max_cpu);

    // Calculate averages
    let avg_memory = samples.iter().map(|(m, _)| *m).sum::<u64>() / samples.len() as u64;
    let avg_cpu = samples.iter().map(|(_, c)| *c).sum::<f32>() / samples.len() as f32;

    println!("  Average memory: {} MB", avg_memory / 1024 / 1024);
    println!("  Average CPU: {:.2}%", avg_cpu);

    // Assertions for resource limits
    assert!(max_memory < 512 * 1024 * 1024, "Memory usage too high: {} MB", max_memory / 1024 / 1024);
    assert!(max_cpu < 80.0, "CPU usage too high: {:.2}%", max_cpu);
    assert!(avg_memory < 256 * 1024 * 1024, "Average memory too high: {} MB", avg_memory / 1024 / 1024);
}

fn get_process_memory(system: &System, process_name: &str) -> u64 {
    system.processes()
        .values()
        .find(|proc| proc.name().contains(process_name))
        .map(|proc| proc.memory())
        .unwrap_or(0) * 1024 // Convert from KB to bytes
}

fn get_process_cpu(system: &System, process_name: &str) -> f32 {
    system.processes()
        .values()
        .find(|proc| proc.name().contains(process_name))
        .map(|proc| proc.cpu_usage())
        .unwrap_or(0.0)
}

async fn run_load_test() {
    // Implement load test that hits the API heavily
    let client = reqwest::Client::new();
    let mut join_set = tokio::task::JoinSet::new();

    for _ in 0..20 {
        let client = client.clone();
        join_set.spawn(async move {
            for _ in 0..100 {
                let _ = client.get("http://localhost:3000/health").send().await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
    }

    while let Some(_) = join_set.join_next().await {}
}
```

## Performance Testing Configuration

### Benchmark Configuration

```toml
# Cargo.toml - benchmark configuration
[[bench]]
name = "api_benchmarks"
harness = false

[[bench]]
name = "storage_benchmarks"
harness = false

[[bench]]
name = "chaincode_benchmarks"
harness = false

[dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
sysinfo = "0.29"
tokio-tungstenite = { version = "0.20", optional = true }
```

### CI/CD Performance Testing

```yaml
# .github/workflows/performance.yml
name: Performance Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: "0 2 * * 0" # Weekly performance regression check

jobs:
  performance:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-perf-${{ hashFiles('**/Cargo.lock') }}

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y build-essential

      - name: Build in release mode
        run: cargo build --release

      - name: Run benchmarks
        run: |
          cargo bench --bench api_benchmarks -- --output-format html
          cargo bench --bench storage_benchmarks -- --output-format html
          cargo bench --bench chaincode_benchmarks -- --output-format html

      - name: Run performance tests
        run: |
          cargo test --release --test performance

      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/

      - name: Performance regression check
        run: |
          # Compare with baseline performance metrics
          ./scripts/check_performance_regression.sh
```

## Performance Monitoring and Alerting

### Continuous Monitoring Setup

```rust
// src/monitoring/performance_metrics.rs
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};
use std::time::Instant;

pub struct PerformanceMetrics {
    pub request_duration: Histogram,
    pub request_count: Counter,
    pub active_connections: Gauge,
    pub memory_usage: Gauge,
    pub cpu_usage: Gauge,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            request_duration: register_histogram!(
                "http_request_duration_seconds",
                "HTTP request duration in seconds"
            ).unwrap(),
            request_count: register_counter!(
                "http_requests_total",
                "Total number of HTTP requests"
            ).unwrap(),
            active_connections: register_gauge!(
                "active_connections",
                "Number of active connections"
            ).unwrap(),
            memory_usage: register_gauge!(
                "memory_usage_bytes",
                "Memory usage in bytes"
            ).unwrap(),
            cpu_usage: register_gauge!(
                "cpu_usage_percent",
                "CPU usage percentage"
            ).unwrap(),
        }
    }

    pub fn record_request_duration(&self, duration: std::time::Duration) {
        self.request_duration.observe(duration.as_secs_f64());
        self.request_count.inc();
    }
}

// Middleware for automatic metrics collection
pub async fn metrics_middleware<B>(
    req: axum::http::Request<B>,
    next: axum::middleware::Next<B>,
) -> axum::response::Response {
    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();

    // Record metrics
    METRICS.record_request_duration(duration);

    response
}
```

This comprehensive performance testing framework ensures the BEACON platform maintains optimal performance under various load conditions and provides early detection of performance regressions.
