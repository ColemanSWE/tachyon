#!/usr/bin/env python3
"""
Simple test script for the Tachyon proxy
"""

import requests
import sys

def test_proxy():
    proxy_url = "http://localhost:8080"
    
    print("Testing Tachyon Proxy...")
    print(f"Proxy URL: {proxy_url}")
    
    try:
        response = requests.get(
            "http://httpbin.org/get",
            proxies={"http": proxy_url},
            timeout=10
        )
        
        print(f"✅ Success! Status: {response.status_code}")
        print(f"Response size: {len(response.content)} bytes")
        return True
        
    except requests.exceptions.ProxyError as e:
        print(f"❌ Proxy error: {e}")
        print("Make sure the proxy is running with: cargo run")
        return False
        
    except requests.exceptions.RequestException as e:
        print(f"❌ Request error: {e}")
        return False

if __name__ == "__main__":
    success = test_proxy()
    sys.exit(0 if success else 1) 