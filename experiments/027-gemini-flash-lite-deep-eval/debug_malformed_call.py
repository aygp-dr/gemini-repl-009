#!/usr/bin/env python3
"""Debug script to test MALFORMED_FUNCTION_CALL issue"""

import os
import json
import requests
from datetime import datetime

API_KEY = os.environ.get('GEMINI_API_KEY')
if not API_KEY:
    print("Error: GEMINI_API_KEY not set")
    exit(1)

# Test with a simple write_file request
test_request = {
    "contents": [{
        "role": "user",
        "parts": [{
            "text": "Create a file hello.py with the content: print('Hello World')"
        }]
    }],
    "tools": [{
        "functionDeclarations": [{
            "name": "write_file",
            "description": "Write content to a file",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to write"
                    },
                    "content": {
                        "type": "string", 
                        "description": "Content to write"
                    }
                },
                "required": ["path", "content"]
            }
        }]
    }]
}

url = f"https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-lite:generateContent?key={API_KEY}"

print(f"Testing at {datetime.now()}")
print("Request:", json.dumps(test_request, indent=2))
print("\n" + "="*50 + "\n")

response = requests.post(url, json=test_request)
print(f"Status: {response.status_code}")
print("Response:", json.dumps(response.json(), indent=2))

# Check if we get MALFORMED_FUNCTION_CALL
if response.status_code == 200:
    resp_json = response.json()
    if 'candidates' in resp_json:
        for candidate in resp_json['candidates']:
            if candidate.get('finishReason') == 'MALFORMED_FUNCTION_CALL':
                print("\n⚠️  MALFORMED_FUNCTION_CALL detected!")
                print("This confirms the issue with write_file function calls")