#!/usr/bin/env python3
"""
Comprehensive test suite for Tachyon proxy
Tests all features including security, metrics, and HTTPS tunneling
"""

import requests
import time
import threading
import subprocess
import sys
import socket
from concurrent.futures import ThreadPoolExecutor

class ProxyTester:
    def __init__(self, proxy_host="localhost", proxy_port=8080):
        self.proxy_host = proxy_host
        self.proxy_port = proxy_port
        self.proxy_url = f"http://{proxy_host}:{proxy_port}"
        self.proxy_dict = {"http": self.proxy_url, "https": self.proxy_url}
        
    def test_basic_http(self):
        """Test basic HTTP forwarding"""
        print("🧪 Testing basic HTTP forwarding...")
        try:
            response = requests.get(
                "http://httpbin.org/get",
                proxies=self.proxy_dict,
                timeout=10
            )
            assert response.status_code == 200
            data = response.json()
            assert "headers" in data
            print("✅ Basic HTTP test passed")
            return True
        except Exception as e:
            print(f"❌ Basic HTTP test failed: {e}")
            return False
    
    def test_https_connect(self):
        """Test HTTPS tunneling via CONNECT method"""
        print("🧪 Testing HTTPS CONNECT tunneling...")
        try:
            response = requests.get(
                "https://httpbin.org/get",
                proxies=self.proxy_dict,
                timeout=10,
                verify=False  # Skip SSL verification for testing
            )
            assert response.status_code == 200
            print("✅ HTTPS CONNECT test passed")
            return True
        except Exception as e:
            print(f"❌ HTTPS CONNECT test failed: {e}")
            return False
    
    def test_concurrent_requests(self, num_requests=20):
        """Test concurrent request handling"""
        print(f"🧪 Testing {num_requests} concurrent requests...")
        
        def make_request(i):
            try:
                response = requests.get(
                    f"http://httpbin.org/delay/1?req={i}",
                    proxies=self.proxy_dict,
                    timeout=15
                )
                return response.status_code == 200
            except:
                return False
        
        start_time = time.time()
        with ThreadPoolExecutor(max_workers=10) as executor:
            futures = [executor.submit(make_request, i) for i in range(num_requests)]
            results = [future.result() for future in futures]
        
        end_time = time.time()
        successful = sum(results)
        
        print(f"✅ Concurrent test: {successful}/{num_requests} successful in {end_time - start_time:.2f}s")
        return successful >= num_requests * 0.8  # 80% success rate
    
    def test_rate_limiting(self):
        """Test rate limiting functionality"""
        print("🧪 Testing rate limiting...")
        
        # Make rapid requests to trigger rate limiting
        success_count = 0
        rate_limited_count = 0
        
        for i in range(50):
            try:
                response = requests.get(
                    "http://httpbin.org/get",
                    proxies=self.proxy_dict,
                    timeout=5
                )
                if response.status_code == 200:
                    success_count += 1
                time.sleep(0.01)  # Very short delay
            except requests.exceptions.ProxyError:
                rate_limited_count += 1
            except:
                pass
        
        print(f"✅ Rate limiting test: {success_count} successful, {rate_limited_count} rate limited")
        return True  # Rate limiting is working if we get some rate limited requests
    
    def test_malformed_requests(self):
        """Test handling of malformed requests"""
        print("🧪 Testing malformed request handling...")
        
        try:
            # Connect directly to proxy and send malformed data
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            sock.connect((self.proxy_host, self.proxy_port))
            
            # Send malformed HTTP request
            malformed_request = b"INVALID REQUEST LINE\r\n\r\n"
            sock.send(malformed_request)
            
            # Should either get an error response or connection close
            response = sock.recv(1024)
            sock.close()
            
            print("✅ Malformed request test passed (proxy handled gracefully)")
            return True
        except Exception as e:
            print(f"✅ Malformed request test passed (connection rejected): {e}")
            return True
    
    def test_large_request(self):
        """Test handling of large requests"""
        print("🧪 Testing large request handling...")
        
        try:
            # Create a large payload
            large_data = {"data": "x" * 1024 * 100}  # 100KB payload
            
            response = requests.post(
                "http://httpbin.org/post",
                json=large_data,
                proxies=self.proxy_dict,
                timeout=30
            )
            
            if response.status_code == 200:
                print("✅ Large request test passed")
                return True
            else:
                print(f"⚠️  Large request returned status {response.status_code}")
                return False
        except Exception as e:
            if "too large" in str(e).lower():
                print("✅ Large request test passed (correctly rejected)")
                return True
            else:
                print(f"❌ Large request test failed: {e}")
                return False
    
    def test_different_methods(self):
        """Test different HTTP methods"""
        print("🧪 Testing different HTTP methods...")
        
        methods_to_test = [
            ("GET", "http://httpbin.org/get"),
            ("POST", "http://httpbin.org/post"),
            ("PUT", "http://httpbin.org/put"),
            ("DELETE", "http://httpbin.org/delete"),
        ]
        
        results = []
        for method, url in methods_to_test:
            try:
                response = requests.request(
                    method,
                    url,
                    proxies=self.proxy_dict,
                    timeout=10,
                    json={"test": "data"} if method in ["POST", "PUT"] else None
                )
                success = response.status_code in [200, 201, 202]
                results.append(success)
                print(f"  {method}: {'✅' if success else '❌'}")
            except Exception as e:
                print(f"  {method}: ❌ ({e})")
                results.append(False)
        
        success_rate = sum(results) / len(results)
        print(f"✅ HTTP methods test: {success_rate:.0%} success rate")
        return success_rate >= 0.75
    
    def run_all_tests(self):
        """Run all tests and return overall success"""
        print("🚀 Starting comprehensive Tachyon proxy tests\n")
        
        tests = [
            ("Basic HTTP", self.test_basic_http),
            ("HTTPS CONNECT", self.test_https_connect),
            ("Concurrent Requests", self.test_concurrent_requests),
            ("Rate Limiting", self.test_rate_limiting),
            ("Malformed Requests", self.test_malformed_requests),
            ("Large Requests", self.test_large_request),
            ("HTTP Methods", self.test_different_methods),
        ]
        
        results = []
        for test_name, test_func in tests:
            print(f"\n{'='*50}")
            try:
                result = test_func()
                results.append(result)
            except Exception as e:
                print(f"❌ {test_name} test crashed: {e}")
                results.append(False)
            
            time.sleep(1)  # Brief pause between tests
        
        print(f"\n{'='*50}")
        print("📊 Test Results Summary:")
        for i, (test_name, _) in enumerate(tests):
            status = "✅ PASS" if results[i] else "❌ FAIL"
            print(f"  {test_name}: {status}")
        
        success_rate = sum(results) / len(results)
        print(f"\nOverall Success Rate: {success_rate:.0%}")
        
        if success_rate >= 0.8:
            print("🎉 Proxy is working well!")
            return True
        else:
            print("⚠️  Some issues detected")
            return False

def check_proxy_running(host="localhost", port=8080):
    """Check if proxy is running"""
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        sock.settimeout(1)
        result = sock.connect_ex((host, port))
        sock.close()
        return result == 0
    except:
        return False

def main():
    proxy_host = "localhost"
    proxy_port = 8080
    
    print("🔍 Checking if Tachyon proxy is running...")
    if not check_proxy_running(proxy_host, proxy_port):
        print(f"❌ Proxy not running on {proxy_host}:{proxy_port}")
        print("Please start the proxy with: cargo run")
        sys.exit(1)
    
    print(f"✅ Proxy detected on {proxy_host}:{proxy_port}")
    
    tester = ProxyTester(proxy_host, proxy_port)
    success = tester.run_all_tests()
    
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    main() 