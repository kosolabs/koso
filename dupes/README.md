# Koso Dupes API Client

A Python client library for interacting with the Koso dupes REST API using httpx. This client provides an asynchronous interface for managing duplicate detection candidates in Koso projects.

## Features

- List duplicate candidates for a project
- Create new duplicate candidates
- Get individual duplicate candidates
- Update duplicate resolution status (mark as duplicate, not duplicate, or clear resolution)
- Async interface with context manager support
- Comprehensive error handling
- CLI tool for testing and administration

## Installation

The client requires Python 3.13+ and httpx. Install dependencies using uv:

```bash
uv sync
```

## Usage

### Async Client

```python
import asyncio
from dupes_client import KosoDupesClient, CreateDupeRequest, KosoDupesAPIError

async def main():
    async with KosoDupesClient(
        base_url="http://localhost:3000",
        auth_token="your_bearer_token"
    ) as client:
        # List all dupes for a project
        dupes = await client.list_dupes("project_id")
        print(f"Found {len(dupes)} duplicates")

        # Create a new dupe candidate
        create_request = CreateDupeRequest(
            task_1_id="task-abc-123",
            task_2_id="task-def-456",
            similarity=0.85
        )
        new_dupe = await client.create_dupe("project_id", create_request)

        # Mark as duplicate
        await client.mark_as_duplicate("project_id", new_dupe.dupe_id)

        # Mark as not duplicate
        await client.mark_as_not_duplicate("project_id", new_dupe.dupe_id)

        # Clear resolution
        await client.clear_resolution("project_id", new_dupe.dupe_id)

asyncio.run(main())
```

## API Endpoints

The client interacts with these REST endpoints:

- `GET /api/projects/{project_id}/dupes` - List all dupes for a project
- `POST /api/projects/{project_id}/dupes` - Create a new dupe candidate
- `GET /api/projects/{project_id}/dupes/{dupe_id}` - Get individual dupe
- `PATCH /api/projects/{project_id}/dupes/{dupe_id}` - Update dupe resolution

## Data Models

### DedupeCandidate

Represents a duplicate candidate with the following fields:

- `dupe_id`: Unique identifier for the duplicate candidate
- `project_id`: ID of the project containing the tasks
- `task_1_id`: ID of the first task
- `task_2_id`: ID of the second task
- `similarity`: Decimal similarity score (0.0 to 1.0)
- `detected_at`: Timestamp when the duplicate was detected
- `resolution`: Optional bool indicating if it's a duplicate (True), not a duplicate (False), or unresolved (None)
- `resolved_at`: Optional timestamp when resolution was set
- `resolved_by`: Optional email of user who set the resolution

### CreateDupeRequest

For creating new duplicate candidates:

- `task_1_id`: ID of the first task
- `task_2_id`: ID of the second task
- `similarity`: Similarity score between 0.0 and 1.0

## Error Handling

The client raises `KosoDupesAPIError` for API errors:

```python
try:
    dupe = await client.get_dupe("project_id", "invalid_dupe_id")
except KosoDupesAPIError as e:
    print(f"API Error {e.status_code}: {e}")
    print(f"Error code: {e.error_code}")  # e.g., "SAME_TASK_IDS", "INVALID_SIMILARITY"
```

Common error codes:

- `SAME_TASK_IDS`: Attempt to create dupe with identical task IDs
- `INVALID_SIMILARITY`: Similarity not between 0.0 and 1.0
- 404: Duplicate candidate or project not found
- 403: Insufficient permissions to access project

## CLI Tool

The package includes a CLI tool for testing and administration:

```bash
# Set environment variables
export KOSO_AUTH_TOKEN="your_bearer_token"
export KOSO_BASE_URL="http://localhost:3000"  # optional

# List dupes
uv run python cli.py list PROJECT_ID

# Create dupe
uv run python cli.py create PROJECT_ID TASK1_ID TASK2_ID 0.85

# Get individual dupe
uv run python cli.py get PROJECT_ID DUPE_ID

# Mark as duplicate
uv run python cli.py mark-duplicate PROJECT_ID DUPE_ID

# Mark as not duplicate
uv run python cli.py mark-not-duplicate PROJECT_ID DUPE_ID

# Clear resolution
uv run python cli.py clear-resolution PROJECT_ID DUPE_ID
```

## Authentication

All API endpoints require authentication via Bearer token. Set your token when initializing the client:

```python
client = KosoDupesClient(auth_token="your_bearer_token_here")
```

Or set the `KOSO_AUTH_TOKEN` environment variable when using the CLI.

## Development

The project uses uv for dependency management. To set up for development:

```bash
# Install dependencies
uv sync

# Run CLI commands
uv run python cli.py --help

# Run example usage
uv run python examples.py
```

## Requirements

- Python 3.13+
- httpx >= 0.28.1
- dataclasses (built-in)
- decimal (built-in)
- datetime (built-in)

## License

This client is part of the Koso project. See the main project LICENSE file for details.
