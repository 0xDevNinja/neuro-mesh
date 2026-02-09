"""Tests for the reference validator."""

import importlib
import sys
from pathlib import Path

# Ensure validator module is importable when running tests from repo root.
sys.path.append(str(Path(__file__).resolve().parent.parent))

def test_validator_imports():
    """The validator module should import without errors."""
    importlib.import_module("validator")