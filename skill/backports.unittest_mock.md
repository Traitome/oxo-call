---
name: backports.unittest_mock
category: Testing / Mocking Framework
description: Provides the unittest.mock module from Python 3.3+ as a backport for Python 2.6+ and Python 3.1+, enabling creation of mock objects, patch decorators, and test spies for isolated unit testing.
tags:
- testing
- mock
- unittest
- backport
- python-2
- python-3
- test-isolation
- stubs
- spies
author: AI-generated
source_url: https://pypi.org/project/backports.unittest_mock/
---

## Concepts

- **Mock Object Creation**: The package provides `Mock` and `MagicMock` classes that create callable objects with configurable return values, side effects, and attribute assignments, allowing you to replace real objects in tests without dependencies on external services or databases.

- **Patch Decorator (`@patch`)**: The `@patch` decorator temporarily replaces attributes or objects in a module or class with mock instances during test execution, automatically restoring the original value after the test completes, which is essential for isolating unit-under-test from its dependencies.

- **Python Version Compatibility**: This backport enables projects supporting Python 2.6+, 3.0+, 3.1+, and 3.2+ to use the same mock API as Python 3.3+ standard library, ensuring consistent testing across multiple Python versions without conditional imports or vendor copies.

- **Spy Functionality**: Using `Mock(wraps=real_object)` creates a spy that proxies calls to the wrapped object while recording all invocations, allowing you to verify interaction patterns (arguments, call counts) without disabling the actual behavior.

## Pitfalls

- **Conflicting with Standard Library**: In Python 3.3+, the standard library already includes `unittest.mock`. Importing `backports.unittest_mock` in Python 3.3+ may cause confusion or duplicate imports; ensure your test runner uses the correct version or conditional imports based on Python version.

- **Forgetting Cleanup with Nested Patches**: Nesting multiple `@patch` decorators without proper ordering can lead to difficult-to-debug issues where inner mocks are not properly restored; always declare patches in the same order as decorator stacking (bottom-up).

- **Mutable Default Mock Attributes**: Directly modifying a mock's return value or attributes inside a test without resetting can cause test pollution, where subsequent tests inherit state from previous tests, leading to flaky test failures that are hard to reproduce.

- **Using Mock in Production Code**: mocks are designed for testing only; accidentally leaving mock patches active in production code will cause runtime failures because the real implementations are replaced with inert mock objects.

## Examples

### Basic Mock Object Creation
**Args:** `from backports.unittest_mock import Mock`
**Explanation:** Creates a new mock object that can be assigned attributes and configured with return values, used to simulate dependencies during testing.

### Isolating a Function Call
**Args:** `from backports.unittest_mock import patch`
**Args:** `@patch('module.ClassName.method_name')`
**Explanation:** Temporarily replaces the specified method with a mock during the test, allowing you to control its return value or verify it was called without hitting the actual implementation.

### Creating a Mock with Predefined Return Value
**Args:** `mock_obj = Mock(return_value={})`
**Args:** `result = mock_obj()`
**Explanation:** Configures a mock to return a specific value (e.g., an empty dict) when called, useful for simulating function responses without executing real logic.

### Verifying Method was Called with Arguments
**Args:** `mock_service.process_data.assert_called_once_with({'key': 'value'})`
**Explanation:** Asserts that the mock was invoked exactly once with the specified arguments, enabling verification of correct parameter passing without checking actual side effects.

### Creating a Spy to Proxy Real Object
**Args:** `from backports.unittest_mock import Mock`
**Args:** `real_api = SomeRealClass()`
**Args:** `spy_api = Mock(wraps=real_api)`
**Explanation:** Creates a spy that wraps a real object, forwarding calls to it while recording all invocations, allowing you to test both behavior and interaction patterns.

### Simulating Exception Raising
**Args:** `mock_obj = Mock()`
**Args:** `mock_obj.side_effect = ValueError('invalid input')`
**Args:** `with self.assertRaises(ValueError): mock_obj()`
**Configures the mock to raise a specific exception when called, enabling you to test error handling code paths reliably.

### Mocking a Class in Module Scope
**Args:** `@patch('mymodule.DatabaseConnection')`
**Args:** `def test_query(self, mock_db_cls):`
**Explanation:** Replaces a class at its location in the module-under-test (not where it's imported), ensuring all references to that class within the module use the mock during the test.

### Chaining Multiple Patches
**Args:** `@patch('os.path.exists')`
**Args:** `@patch('os.makedirs')`
**Args:** `def test_create_dir(self, mock_makedirs, mock_exists):`
**Explanation:** Multiple decorators create layered mocks, applied in reverse order (mock_exists applied first), allowing you to mock multiple system interactions in a single test function.

### Async Mock Configuration
**Args:** `async_mock = AsyncMock(return_value='result')`
**Args:** `result = await async_mock()`
**Explanation:** Creates an async mock that can be awaited, essential for testing async/await code in Python 3.5+ environments using this backport.