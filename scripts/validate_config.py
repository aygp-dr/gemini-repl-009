#!/usr/bin/env python3
"""
Validate Gemini REPL configuration files against the specification.

Usage:
    python validate_config.py [config.toml]
    python validate_config.py --generate-schema > schema.json
"""

import sys
import os
import re
import toml
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass
from enum import Enum

class ValidationLevel(Enum):
    ERROR = "ERROR"
    WARNING = "WARNING"
    INFO = "INFO"

@dataclass
class ValidationIssue:
    level: ValidationLevel
    path: str
    message: str

class ConfigValidator:
    """Validates Gemini REPL configuration files."""
    
    # Valid models
    VALID_MODELS = {
        "gemini-1.5-flash",
        "gemini-1.5-pro", 
        "gemini-2.0-flash-exp",
        "gemini-pro",
        "gemini-pro-vision"
    }
    
    # Valid log levels
    VALID_LOG_LEVELS = {"error", "warn", "info", "debug", "trace"}
    
    # Valid log formats
    VALID_LOG_FORMATS = {"json", "pretty", "compact"}
    
    # Valid themes
    VALID_THEMES = {"default", "minimal", "solarized", "dracula"}
    
    # Valid spinner styles
    VALID_SPINNER_STYLES = {"dots", "line", "simple"}
    
    # Size regex pattern
    SIZE_PATTERN = re.compile(r'^\d+(\.\d+)?\s*(B|KB?|MB?|GB?)?$', re.IGNORECASE)
    
    # Dangerous commands to warn about
    DANGEROUS_COMMANDS = {
        "rm", "dd", "mkfs", "fdisk", "shred", 
        "chmod", "chown", ">", ">>", "|"
    }
    
    def __init__(self):
        self.issues: List[ValidationIssue] = []
    
    def validate_file(self, config_path: Path) -> Tuple[bool, List[ValidationIssue]]:
        """Validate a configuration file."""
        self.issues = []
        
        # Check file exists
        if not config_path.exists():
            self.add_error("", f"Configuration file not found: {config_path}")
            return False, self.issues
        
        # Parse TOML
        try:
            with open(config_path, 'r') as f:
                config = toml.load(f)
        except toml.TomlDecodeError as e:
            self.add_error("", f"Invalid TOML syntax: {e}")
            return False, self.issues
        except Exception as e:
            self.add_error("", f"Error reading file: {e}")
            return False, self.issues
        
        # Validate structure
        self.validate_structure(config)
        
        # Validate each section
        if "api" in config:
            self.validate_api(config["api"])
        if "repl" in config:
            self.validate_repl(config["repl"])
        if "logging" in config:
            self.validate_logging(config["logging"])
        if "tools" in config:
            self.validate_tools(config["tools"])
        if "session" in config:
            self.validate_session(config["session"])
        if "ui" in config:
            self.validate_ui(config["ui"])
        if "response" in config:
            self.validate_response(config["response"])
        if "network" in config:
            self.validate_network(config["network"])
        if "security" in config:
            self.validate_security(config["security"])
        if "debug" in config:
            self.validate_debug(config["debug"])
        if "aliases" in config:
            self.validate_aliases(config["aliases"])
        if "models" in config:
            self.validate_models(config["models"])
        if "prompts" in config:
            self.validate_prompts(config["prompts"])
        if "features" in config:
            self.validate_features(config["features"])
        
        # Cross-validation
        self.cross_validate(config)
        
        # Return results
        has_errors = any(issue.level == ValidationLevel.ERROR for issue in self.issues)
        return not has_errors, self.issues
    
    def validate_structure(self, config: Dict[str, Any]):
        """Validate top-level structure."""
        valid_tables = {
            "api", "repl", "logging", "tools", "session", "ui",
            "response", "network", "security", "debug", "aliases",
            "models", "prompts", "features"
        }
        
        for key in config:
            if key not in valid_tables:
                self.add_warning("", f"Unknown top-level table: {key}")
    
    def validate_api(self, api: Dict[str, Any]):
        """Validate [api] section."""
        # Check for API key (security warning)
        if "api_key" in api:
            self.add_warning("api.api_key", 
                "API key should not be stored in config file. Use GEMINI_API_KEY environment variable.")
        
        # Validate model
        if "model" in api:
            if api["model"] not in self.VALID_MODELS:
                self.add_error("api.model", 
                    f"Invalid model: {api['model']}. Valid models: {', '.join(self.VALID_MODELS)}")
        
        # Validate base_url
        if "base_url" in api:
            if not self.is_valid_url(api["base_url"]):
                self.add_error("api.base_url", "Invalid URL format")
        
        # Validate timeout
        if "timeout" in api:
            self.validate_integer_range("api.timeout", api["timeout"], 1, 300)
        
        # Validate max_retries
        if "max_retries" in api:
            self.validate_integer_range("api.max_retries", api["max_retries"], 0, 10)
        
        # Validate retry_delay
        if "retry_delay" in api:
            self.validate_float_range("api.retry_delay", api["retry_delay"], 0.1, 60.0)
    
    def validate_repl(self, repl: Dict[str, Any]):
        """Validate [repl] section."""
        # Validate history_size
        if "history_size" in repl:
            self.validate_integer_range("repl.history_size", repl["history_size"], 0, 100000)
        
        # Validate auto_save_interval
        if "auto_save_interval" in repl:
            self.validate_integer_range("repl.auto_save_interval", repl["auto_save_interval"], 0, 3600)
        
        # Validate booleans
        bool_fields = ["colored_prompt", "welcome_banner", "vi_mode", "multiline_mode"]
        for field in bool_fields:
            if field in repl:
                self.validate_boolean(f"repl.{field}", repl[field])
    
    def validate_logging(self, logging: Dict[str, Any]):
        """Validate [logging] section."""
        # Validate level
        if "level" in logging:
            if logging["level"] not in self.VALID_LOG_LEVELS:
                self.add_error("logging.level",
                    f"Invalid log level: {logging['level']}. Valid levels: {', '.join(self.VALID_LOG_LEVELS)}")
        
        # Validate format
        if "format" in logging:
            if logging["format"] not in self.VALID_LOG_FORMATS:
                self.add_error("logging.format",
                    f"Invalid log format: {logging['format']}. Valid formats: {', '.join(self.VALID_LOG_FORMATS)}")
        
        # Validate max_file_size
        if "max_file_size" in logging:
            if not self.is_valid_size(logging["max_file_size"]):
                self.add_error("logging.max_file_size", "Invalid size format. Use: 10MB, 1GB, etc.")
        
        # Validate max_files
        if "max_files" in logging:
            self.validate_integer_range("logging.max_files", logging["max_files"], 1, 100)
    
    def validate_tools(self, tools: Dict[str, Any]):
        """Validate [tools] section."""
        # Validate max_file_size
        if "max_file_size" in tools:
            if not self.is_valid_size(tools["max_file_size"]):
                self.add_error("tools.max_file_size", "Invalid size format")
        
        # Validate allowed_extensions
        if "allowed_extensions" in tools:
            if not isinstance(tools["allowed_extensions"], list):
                self.add_error("tools.allowed_extensions", "Must be an array of strings")
            else:
                for ext in tools["allowed_extensions"]:
                    if not isinstance(ext, str) or not ext.startswith("."):
                        self.add_error("tools.allowed_extensions", 
                            f"Invalid extension format: {ext}. Must start with '.'")
        
        # Validate commands sub-table
        if "commands" in tools:
            self.validate_tools_commands(tools["commands"])
    
    def validate_tools_commands(self, commands: Dict[str, Any]):
        """Validate [tools.commands] section."""
        if "timeout" in commands:
            self.validate_integer_range("tools.commands.timeout", commands["timeout"], 1, 300)
        
        if "allowed_commands" in commands:
            if not isinstance(commands["allowed_commands"], list):
                self.add_error("tools.commands.allowed_commands", "Must be an array of strings")
            else:
                for cmd in commands["allowed_commands"]:
                    # Check for dangerous commands
                    for dangerous in self.DANGEROUS_COMMANDS:
                        if dangerous in cmd:
                            self.add_warning("tools.commands.allowed_commands",
                                f"Potentially dangerous command: {cmd}")
    
    def validate_session(self, session: Dict[str, Any]):
        """Validate [session] section."""
        if "max_context_size" in session:
            self.validate_integer_range("session.max_context_size", session["max_context_size"], 
                                       1000, 1000000)
        
        if "prune_strategy" in session:
            valid_strategies = {"oldest", "summarize", "smart"}
            if session["prune_strategy"] not in valid_strategies:
                self.add_error("session.prune_strategy",
                    f"Invalid strategy. Valid: {', '.join(valid_strategies)}")
    
    def validate_ui(self, ui: Dict[str, Any]):
        """Validate [ui] section."""
        if "theme" in ui:
            if ui["theme"] not in self.VALID_THEMES:
                self.add_warning("ui.theme", 
                    f"Unknown theme: {ui['theme']}. Valid themes: {', '.join(self.VALID_THEMES)}")
        
        if "spinner_style" in ui:
            if ui["spinner_style"] not in self.VALID_SPINNER_STYLES:
                self.add_error("ui.spinner_style",
                    f"Invalid spinner style. Valid: {', '.join(self.VALID_SPINNER_STYLES)}")
        
        if "max_width" in ui:
            self.validate_integer_range("ui.max_width", ui["max_width"], 40, 200)
    
    def validate_response(self, response: Dict[str, Any]):
        """Validate [response] section."""
        if "temperature" in response:
            self.validate_float_range("response.temperature", response["temperature"], 0.0, 2.0)
        
        if "max_tokens" in response:
            self.validate_integer_range("response.max_tokens", response["max_tokens"], 1, 100000)
        
        if "format" in response:
            valid_formats = {"auto", "plain", "markdown", "json"}
            if response["format"] not in valid_formats:
                self.add_error("response.format",
                    f"Invalid format. Valid: {', '.join(valid_formats)}")
    
    def validate_network(self, network: Dict[str, Any]):
        """Validate [network] section."""
        if "proxy_url" in network:
            if not self.is_valid_url(network["proxy_url"]):
                self.add_error("network.proxy_url", "Invalid proxy URL")
        
        if "timeout_connect" in network:
            self.validate_integer_range("network.timeout_connect", network["timeout_connect"], 1, 60)
        
        if "timeout_read" in network:
            self.validate_integer_range("network.timeout_read", network["timeout_read"], 1, 300)
    
    def validate_security(self, security: Dict[str, Any]):
        """Validate [security] section."""
        bool_fields = ["mask_api_key", "audit_tools", "validate_ssl", "sanitize_logs"]
        for field in bool_fields:
            if field in security:
                self.validate_boolean(f"security.{field}", security[field])
    
    def validate_debug(self, debug: Dict[str, Any]):
        """Validate [debug] section."""
        bool_fields = ["show_raw_api_calls", "save_recordings", "verbose_errors"]
        for field in bool_fields:
            if field in debug:
                self.validate_boolean(f"debug.{field}", debug[field])
        
        if "mock_delay_ms" in debug:
            self.validate_integer_range("debug.mock_delay_ms", debug["mock_delay_ms"], 0, 5000)
    
    def validate_aliases(self, aliases: Dict[str, Any]):
        """Validate [aliases] section."""
        for alias, command in aliases.items():
            if not isinstance(command, str):
                self.add_error(f"aliases.{alias}", "Alias target must be a string")
    
    def validate_models(self, models: Dict[str, Any]):
        """Validate [models.*] sections."""
        for model_name, config in models.items():
            if "temperature" in config:
                self.validate_float_range(f"models.{model_name}.temperature", 
                                        config["temperature"], 0.0, 2.0)
            if "max_tokens" in config:
                self.validate_integer_range(f"models.{model_name}.max_tokens", 
                                          config["max_tokens"], 1, 100000)
    
    def validate_prompts(self, prompts: Dict[str, Any]):
        """Validate [prompts] section."""
        for name, prompt in prompts.items():
            if not isinstance(prompt, str):
                self.add_error(f"prompts.{name}", "Prompt must be a string")
    
    def validate_features(self, features: Dict[str, Any]):
        """Validate [features] section."""
        for feature, enabled in features.items():
            self.validate_boolean(f"features.{feature}", enabled)
    
    def cross_validate(self, config: Dict[str, Any]):
        """Perform cross-field validation."""
        # If log_requests is true, should have log file
        if config.get("logging", {}).get("log_requests", False):
            if not config.get("logging", {}).get("file"):
                self.add_warning("logging", 
                    "log_requests is true but no log file specified")
        
        # If auto_save is true, need valid session dir
        if config.get("session", {}).get("auto_save", False):
            if not config.get("session", {}).get("default_dir"):
                self.add_warning("session",
                    "auto_save is true but no default_dir specified")
    
    # Helper methods
    
    def validate_integer_range(self, path: str, value: Any, min_val: int, max_val: int):
        """Validate integer is within range."""
        if not isinstance(value, int):
            self.add_error(path, f"Must be an integer")
        elif value < min_val or value > max_val:
            self.add_error(path, f"Must be between {min_val} and {max_val}")
    
    def validate_float_range(self, path: str, value: Any, min_val: float, max_val: float):
        """Validate float is within range."""
        if not isinstance(value, (int, float)):
            self.add_error(path, f"Must be a number")
        elif value < min_val or value > max_val:
            self.add_error(path, f"Must be between {min_val} and {max_val}")
    
    def validate_boolean(self, path: str, value: Any):
        """Validate boolean value."""
        if not isinstance(value, bool):
            self.add_error(path, "Must be true or false")
    
    def is_valid_url(self, url: str) -> bool:
        """Check if URL is valid."""
        return url.startswith(("http://", "https://"))
    
    def is_valid_size(self, size: str) -> bool:
        """Check if size format is valid."""
        return bool(self.SIZE_PATTERN.match(str(size)))
    
    def add_error(self, path: str, message: str):
        """Add validation error."""
        self.issues.append(ValidationIssue(ValidationLevel.ERROR, path, message))
    
    def add_warning(self, path: str, message: str):
        """Add validation warning."""
        self.issues.append(ValidationIssue(ValidationLevel.WARNING, path, message))
    
    def add_info(self, path: str, message: str):
        """Add validation info."""
        self.issues.append(ValidationIssue(ValidationLevel.INFO, path, message))


def main():
    """CLI entry point."""
    if len(sys.argv) > 1 and sys.argv[1] == "--generate-schema":
        # Generate JSON schema (simplified example)
        schema = {
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "api": {
                    "type": "object",
                    "properties": {
                        "model": {"type": "string", "enum": list(ConfigValidator.VALID_MODELS)},
                        "timeout": {"type": "integer", "minimum": 1, "maximum": 300}
                    }
                }
            }
        }
        import json
        print(json.dumps(schema, indent=2))
        return
    
    # Validate config file
    if len(sys.argv) < 2:
        config_path = Path.home() / ".gemini-repl" / "config.toml"
    else:
        config_path = Path(sys.argv[1])
    
    validator = ConfigValidator()
    is_valid, issues = validator.validate_file(config_path)
    
    # Print results
    if not issues:
        print(f"✅ Configuration file is valid: {config_path}")
        return 0
    
    # Group issues by level
    errors = [i for i in issues if i.level == ValidationLevel.ERROR]
    warnings = [i for i in issues if i.level == ValidationLevel.WARNING]
    infos = [i for i in issues if i.level == ValidationLevel.INFO]
    
    # Print issues
    for issue in errors:
        print(f"❌ ERROR: {issue.path}: {issue.message}")
    
    for issue in warnings:
        print(f"⚠️  WARNING: {issue.path}: {issue.message}")
    
    for issue in infos:
        print(f"ℹ️  INFO: {issue.path}: {issue.message}")
    
    # Summary
    print(f"\nSummary: {len(errors)} errors, {len(warnings)} warnings, {len(infos)} info messages")
    
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
