#!/usr/bin/env python3
"""
Simple benchmark script for Tachyon proxy
"""

import requests
import time
import threading
from concurrent.futures import ThreadPoolExecutor
import sys

def make_request(proxy_url, url, timeout=10):
    """Make a single request through the proxy"""
    start_time = time.time()
    try:
        response = requests.get(
            url,
            proxies={"http": proxy_url},
            timeout=timeout
        )
        end_time = time.time()
        return {
            "success": True,
            "status_code": response.status_code,
            "response_time": end_time - start_time,
            "response_size": len(response.content)
        }
    except Exception as e:
        end_time = time.time()
        return {
            "success": False,
            "error": str(e),
            "response_time": end_time - start_time
        }

def benchmark_concurrent(proxy_url, url, num_requests, max_workers):
    """Benchmark with concurrent requests"""
    print(f"Running {num_requests} concurrent requests with {max_workers} workers...")
    
    start_time = time.time()
    
    with ThreadPoolExecutor(max_workers=max_workers) as executor:
        futures = [executor.submit(make_request, proxy_url, url) for _ in range(num_requests)]
        results = [future.result() for future in futures]
    
    end_time = time.time()
    total_time = end_time - start_time
    
    successful_requests = [r for r in results if r["success"]]
    failed_requests = [r for r in results if not r["success"]]
    
    if successful_requests:
        response_times = [r["response_time"] for r in successful_requests]
        avg_response_time = sum(response_times) / len(response_times)
        
        print(f"✅ Successful requests: {len(successful_requests)}")
        print(f"❌ Failed requests: {len(failed_requests)}")
        print(f"📊 Average response time: {avg_response_time:.3f}s")
        print(f"📊 Total time: {total_time:.3f}s")
        print(f"📊 Requests per second: {len(successful_requests) / total_time:.2f}")

def main():
    proxy_url = "http://localhost:8080"
    test_url = "http://httpbin.org/get"
    
    print("🚀 Tachyon Proxy Benchmark")
    print(f"Proxy: {proxy_url}")
    print(f"Target: {test_url}")
    print("-" * 50)
    
    # Test single request first
    print("Testing single request...")
    result = make_request(proxy_url, test_url)
    if result["success"]:
        print(f"✅ Single request: {result['response_time']:.3f}s")
    else:
        print(f"❌ Single request failed: {result['error']}")
        print("Make sure the proxy is running!")
        sys.exit(1)
    
    print("\n" + "=" * 50)
    
    # Test different concurrency levels
    test_configs = [
        (10, 5),    # 10 requests, 5 workers
        (50, 10),   # 50 requests, 10 workers
    ]
    
    for num_requests, max_workers in test_configs:
        print(f"\n🧪 Test: {num_requests} requests, {max_workers} workers")
        benchmark_concurrent(proxy_url, test_url, num_requests, max_workers)
        time.sleep(1)

if __name__ == "__main__":
    main() 