/// System Bridge Demo
///
/// This example demonstrates all functions of the SYS bridge module.
/// It shows how to retrieve platform information, architecture, version,
/// and comprehensive environment info.

use host_bridge::{HostBridge, SysBridge};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("=== Frame Framework SYS Bridge Demo ===\n");

    demo_direct_api().await;
    println!();
    demo_bridge_api().await;
    println!();
    demo_env_info().await;
    println!();
    demo_error_handling().await;

    println!("\n=== Demo Complete ===");
}

/// Demonstrate direct API access
async fn demo_direct_api() {
    println!("--- Direct API Demo ---");

    let bridge = SysBridge::new();

    // Get platform
    let platform = bridge.get_platform();
    println!("Platform: {}", platform);

    // Get architecture
    let arch = bridge.get_arch();
    println!("Architecture: {}", arch);

    // Get version
    let version = bridge.get_version();
    println!("Framework Version: {}", version);

    // Get process ID
    let pid = bridge.get_process_id();
    println!("Process ID: {}", pid);
}

/// Demonstrate bridge API through JSON calls
async fn demo_bridge_api() {
    println!("--- Bridge API Demo ---");

    let bridge = SysBridge::new();

    // Platform call
    let result = bridge.call("platform", json!({})).await.unwrap();
    println!("Platform (JSON): {}", result);

    // Architecture call
    let result = bridge.call("arch", json!({})).await.unwrap();
    println!("Architecture (JSON): {}", result);

    // Version call
    let result = bridge.call("version", json!({})).await.unwrap();
    println!("Version (JSON): {}", result);
}

/// Demonstrate comprehensive environment info
async fn demo_env_info() {
    println!("--- Environment Info Demo ---");

    let mut bridge = HostBridge::new();

    // Get comprehensive environment info through main bridge
    let result = bridge.call("sys", "env_info", json!({})).await.unwrap();

    if result["ok"] == true {
        let data = &result["data"];

        println!("Complete Environment Information:");
        println!("  Platform:          {}", data["platform"]);
        println!("  Architecture:      {}", data["arch"]);
        println!("  Framework Version: {}", data["version"]);
        println!("  Uptime (seconds):  {}", data["uptime_seconds"]);
        println!("  Process ID:        {}", data["process_id"]);
        println!("  Parent Process ID: {}", data["parent_process_id"]);

        // Wait a bit and check uptime again
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let result2 = bridge.call("sys", "env_info", json!({})).await.unwrap();
        println!("  Uptime after 100ms: {}", result2["data"]["uptime_seconds"]);
    }
}

/// Demonstrate error handling
async fn demo_error_handling() {
    println!("--- Error Handling Demo ---");

    let mut bridge = HostBridge::new();

    // Test 1: Unknown function
    println!("\n1. Testing unknown function:");
    let result = bridge.call("sys", "unknown_function", json!({})).await.unwrap();
    if result["ok"] == false {
        println!("   Error Code: {}", result["err"]["code"]);
        println!("   Message: {}", result["err"]["message"]);
    }

    // Test 2: Invalid exit code (negative)
    println!("\n2. Testing invalid exit code (negative):");
    let result = bridge.call("sys", "exit", json!({"code": -1})).await.unwrap();
    if result["ok"] == false {
        println!("   Error Code: {}", result["err"]["code"]);
        println!("   Message: {}", result["err"]["message"]);
        println!("   Details: {}", result["err"]["details"]);
    }

    // Test 3: Invalid exit code (too large)
    println!("\n3. Testing invalid exit code (too large):");
    let result = bridge.call("sys", "exit", json!({"code": 300})).await.unwrap();
    if result["ok"] == false {
        println!("   Error Code: {}", result["err"]["code"]);
        println!("   Message: {}", result["err"]["message"]);
        println!("   Details: {}", result["err"]["details"]);
    }

    // Test 4: Invalid exit request format
    println!("\n4. Testing invalid exit request format:");
    let result = bridge.call("sys", "exit", json!({"invalid": "field"})).await.unwrap();
    if result["ok"] == false {
        println!("   Error Code: {}", result["err"]["code"]);
        println!("   Message: {}", result["err"]["message"]);
    }

    println!("\n✓ All error cases handled correctly");
}

// Note: We don't demonstrate the actual exit() function because it would
// terminate this demo program. In production, you would use:
//
// let result = bridge.call("sys", "exit", json!({"code": 0})).await;
//
// This would exit the process with code 0 (success).
