#!/usr/bin/env python3
"""
Comprehensive test file for enhanced Python symbol extractor
Tests async functions, type annotations, decorators, inheritance, and more.
"""

from abc import ABC, abstractmethod
from dataclasses import dataclass
from enum import Enum
from typing import List, Dict, Optional, Union, Callable, Any
import asyncio


class ExceptionBase(Exception):
    """Base exception class for testing inheritance detection."""
    pass


class DataProcessingError(ExceptionBase):
    """Custom exception for testing exception class detection."""
    pass


@dataclass
class UserInfo:
    """Data class for testing dataclass decorator detection."""
    name: str
    age: int
    email: Optional[str] = None


class ProcessingStatus(Enum):
    """Enum for testing enum class detection."""
    PENDING = "pending"
    PROCESSING = "processing"
    COMPLETED = "completed"
    FAILED = "failed"


class BaseProcessor(ABC):
    """Abstract base class for testing ABC detection."""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
    
    @abstractmethod
    def process(self, data: List[Any]) -> bool:
        """Abstract method for testing abstractmethod decorator."""
        pass
    
    @classmethod
    def create_default(cls) -> 'BaseProcessor':
        """Class method for testing classmethod decorator."""
        return cls({})
    
    @staticmethod
    def validate_config(config: Dict[str, Any]) -> bool:
        """Static method for testing staticmethod decorator."""
        return isinstance(config, dict)


class DataProcessor(BaseProcessor):
    """Main processor class for testing inheritance and method detection."""
    
    def __init__(self, config: Dict[str, Any], max_retries: int = 3):
        super().__init__(config)
        self.max_retries = max_retries
        self._private_var = "private"
    
    async def process_async(self, data: List[Any]) -> bool:
        """Async method for testing async detection."""
        try:
            result = await self._process_data_async(data)
            return result
        except Exception as e:
            print(f"Error processing async: {e}")
            return False
    
    def process(self, data: List[Any]) -> bool:
        """Sync implementation of abstract method."""
        return len(data) > 0
    
    @property
    def is_configured(self) -> bool:
        """Property method for testing property decorator."""
        return bool(self.config)
    
    @property
    def retry_count(self) -> int:
        """Read-only property for testing property detection."""
        return self.max_retries
    
    def _process_data_sync(self, data: List[Any]) -> bool:
        """Private method for testing private method detection."""
        return True
    
    async def _process_data_async(self, data: List[Any]) -> bool:
        """Private async method for testing async + private detection."""
        await asyncio.sleep(0.1)
        return True
    
    def __str__(self) -> str:
        """Dunder method for testing dunder detection."""
        return f"DataProcessor(config={self.config})"
    
    def __len__(self) -> int:
        """Another dunder method."""
        return len(self.config)


class DataGenerator:
    """Generator class for testing generator function detection."""
    
    def __init__(self, data: List[Any]):
        self.data = data
    
    def generate_batches(self, batch_size: int) -> List[List[Any]]:
        """Generator function for testing yield detection."""
        for i in range(0, len(self.data), batch_size):
            yield self.data[i:i + batch_size]
    
    def process_with_callback(self, data: List[Any], callback: Callable[[Any], bool]) -> List[bool]:
        """Function with callable parameter for testing type hints."""
        results = []
        for item in data:
            try:
                result = callback(item)
                results.append(result)
            except Exception as e:
                results.append(False)
        return results


# Constants for testing constant detection
MAX_BATCH_SIZE = 1000
DEFAULT_TIMEOUT = 30.0
PROCESSOR_VERSION = "1.0.0"


def create_processor(config: Dict[str, Any]) -> DataProcessor:
    """Factory function for testing function type hints."""
    return DataProcessor(config)


async def main() -> None:
    """Main async function for testing."""
    # Test dataclass
    user = UserInfo(name="Alice", age=30, email="alice@example.com")
    
    # Test processor creation
    processor = DataProcessor({"batch_size": 50}, max_retries=5)
    
    # Test async processing
    data = [1, 2, 3, 4, 5]
    result = await processor.process_async(data)
    
    # Test generator
    generator = DataGenerator(data)
    batches = list(generator.generate_batches(2))
    
    # Test property access
    is_ready = processor.is_configured
    retries = processor.retry_count
    
    print(f"Processing result: {result}")
    print(f"Generated {len(batches)} batches")
    print(f"Processor ready: {is_ready}, Max retries: {retries}")


if __name__ == "__main__":
    asyncio.run(main())