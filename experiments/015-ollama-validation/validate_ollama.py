#!/usr/bin/env python3
"""Validate Ollama API responses against expected schema using Pydantic."""

import json
import subprocess
import sys
from datetime import datetime
from typing import List, Optional, Dict, Any

import requests
from pydantic import BaseModel, Field, ValidationError


# Ollama Response Models
class OllamaGenerateResponse(BaseModel):
    """Response from /api/generate endpoint"""
    model: str
    created_at: datetime
    response: str
    done: bool
    context: Optional[List[int]] = None
    total_duration: Optional[int] = None
    load_duration: Optional[int] = None
    prompt_eval_count: Optional[int] = None
    prompt_eval_duration: Optional[int] = None
    eval_count: Optional[int] = None
    eval_duration: Optional[int] = None


class Message(BaseModel):
    """Chat message format"""
    role: str = Field(..., pattern="^(system|user|assistant)$")
    content: str
    

class ChatRequest(BaseModel):
    """Request format for /api/chat endpoint"""
    model: str
    messages: List[Message]
    stream: bool = False
    format: Optional[str] = None
    options: Optional[Dict[str, Any]] = None
    tools: Optional[List[Dict[str, Any]]] = None


class ToolCall(BaseModel):
    """Tool call in response"""
    function: Dict[str, Any]


class ChatMessage(BaseModel):
    """Response message with potential tool calls"""
    role: str
    content: str
    tool_calls: Optional[List[ToolCall]] = None


class ChatResponse(BaseModel):
    """Response from /api/chat endpoint"""
    model: str
    created_at: datetime
    message: ChatMessage
    done: bool
    total_duration: Optional[int] = None
    load_duration: Optional[int] = None
    prompt_eval_count: Optional[int] = None
    prompt_eval_duration: Optional[int] = None
    eval_count: Optional[int] = None
    eval_duration: Optional[int] = None


def test_cli_simple():
    """Test simple CLI interaction"""
    print("\n=== Test 1: CLI Simple Math ===")
    
    try:
        # Run ollama with simple prompt
        result = subprocess.run(
            ["ollama", "run", "llama3.2", "What is 2 + 2? Just give the number."],
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode == 0:
            print(f"✓ CLI Response: {result.stdout.strip()}")
            # Check if response contains "4"
            if "4" in result.stdout:
                print("✓ Correct answer detected")
            else:
                print("✗ Expected '4' in response")
        else:
            print(f"✗ CLI failed: {result.stderr}")
            
    except subprocess.TimeoutExpired:
        print("✗ CLI command timed out")
    except FileNotFoundError:
        print("✗ Ollama CLI not found. Install with: curl -fsSL https://ollama.ai/install.sh | sh")


def test_api_generate():
    """Test /api/generate endpoint"""
    print("\n=== Test 2: API Generate Endpoint ===")
    
    url = "http://localhost:11434/api/generate"
    payload = {
        "model": "llama3.2",
        "prompt": "What is 2 + 2?",
        "stream": False
    }
    
    try:
        response = requests.post(url, json=payload, timeout=30)
        response.raise_for_status()
        
        data = response.json()
        print(f"Raw response keys: {list(data.keys())}")
        
        # Validate with Pydantic
        validated = OllamaGenerateResponse(**data)
        print(f"✓ Valid response structure")
        print(f"  Model: {validated.model}")
        print(f"  Response: {validated.response[:100]}...")
        print(f"  Tokens evaluated: {validated.eval_count}")
        
    except ValidationError as e:
        print(f"✗ Response validation failed:")
        for error in e.errors():
            print(f"  - {error['loc']}: {error['msg']}")
    except requests.exceptions.ConnectionError:
        print("✗ Cannot connect to Ollama. Start with: ollama serve")
    except Exception as e:
        print(f"✗ Request failed: {e}")


def test_api_chat():
    """Test /api/chat endpoint with conversation"""
    print("\n=== Test 3: API Chat Endpoint ===")
    
    url = "http://localhost:11434/api/chat"
    payload = {
        "model": "llama3.2",
        "messages": [
            {"role": "user", "content": "What is 2 + 2?"},
            {"role": "assistant", "content": "2 + 2 equals 4."},
            {"role": "user", "content": "Double it"}
        ],
        "stream": False
    }
    
    try:
        # Validate request
        request_model = ChatRequest(**payload)
        print("✓ Valid request structure")
        
        response = requests.post(url, json=payload, timeout=30)
        response.raise_for_status()
        
        data = response.json()
        
        # Validate response
        validated = ChatResponse(**data)
        print(f"✓ Valid chat response")
        print(f"  Assistant: {validated.message.content[:100]}...")
        
    except ValidationError as e:
        print(f"✗ Validation failed:")
        for error in e.errors():
            print(f"  - {error['loc']}: {error['msg']}")
    except Exception as e:
        print(f"✗ Request failed: {e}")


def test_tool_calling():
    """Test tool calling (if supported by model)"""
    print("\n=== Test 4: Tool Calling ===")
    
    url = "http://localhost:11434/api/chat"
    
    # Define a simple tool
    tools = [{
        "type": "function",
        "function": {
            "name": "get_weather",
            "description": "Get the weather for a location",
            "parameters": {
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "City name"
                    }
                },
                "required": ["location"]
            }
        }
    }]
    
    payload = {
        "model": "llama3.2",  # Note: Most models don't support tools yet
        "messages": [
            {"role": "user", "content": "What's the weather in San Francisco?"}
        ],
        "tools": tools,
        "stream": False
    }
    
    try:
        response = requests.post(url, json=payload, timeout=30)
        response.raise_for_status()
        
        data = response.json()
        validated = ChatResponse(**data)
        
        if validated.message.tool_calls:
            print("✓ Model supports tool calling")
            for tool_call in validated.message.tool_calls:
                print(f"  Tool: {tool_call.function}")
        else:
            print("ℹ Model responded without tool calls (may not support tools)")
            print(f"  Response: {validated.message.content[:100]}...")
            
    except Exception as e:
        print(f"✗ Tool calling test failed: {e}")


def compare_with_gemini():
    """Show differences between Ollama and Gemini API formats"""
    print("\n=== API Format Comparison ===")
    
    print("\nGemini format:")
    gemini_format = {
        "contents": [
            {"role": "user", "parts": [{"text": "Hello"}]}
        ]
    }
    print(json.dumps(gemini_format, indent=2))
    
    print("\nOllama format:")
    ollama_format = {
        "messages": [
            {"role": "user", "content": "Hello"}
        ]
    }
    print(json.dumps(ollama_format, indent=2))
    
    print("\nKey differences:")
    print("- Gemini: 'contents' with 'parts', roles: 'user'/'model'")
    print("- Ollama: 'messages' with 'content', roles: 'user'/'assistant'/'system'")
    print("- Gemini: Structured 'parts' array for multi-modal")
    print("- Ollama: Simple 'content' string")


def save_schemas():
    """Save Pydantic schemas as JSON Schema for documentation"""
    print("\n=== Saving Schemas ===")
    
    schemas = {
        "ollama_generate_response": OllamaGenerateResponse.model_json_schema(),
        "ollama_chat_request": ChatRequest.model_json_schema(),
        "ollama_chat_response": ChatResponse.model_json_schema(),
    }
    
    with open("ollama_schemas.json", "w") as f:
        json.dump(schemas, f, indent=2)
    
    print("✓ Schemas saved to ollama_schemas.json")


if __name__ == "__main__":
    print("Ollama API Validation Test Suite")
    print("================================")
    
    # Check if Ollama is running
    try:
        response = requests.get("http://localhost:11434/api/tags", timeout=5)
        if response.status_code == 200:
            models = response.json().get("models", [])
            print(f"✓ Ollama is running with {len(models)} models")
            if models:
                print(f"  Available: {', '.join(m['name'] for m in models[:3])}...")
        else:
            print("✗ Ollama responded but with unexpected status")
    except requests.exceptions.ConnectionError:
        print("✗ Ollama is not running. Start with: ollama serve")
        print("  Then pull a model: ollama pull llama3.2")
        sys.exit(1)
    
    # Run tests
    test_cli_simple()
    test_api_generate()
    test_api_chat()
    test_tool_calling()
    compare_with_gemini()
    save_schemas()
    
    print("\n✓ Validation complete!")