import argparse
import os

import httpx

HEADERS = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {os.environ['KOSO_AUTH_TOKEN']}",
}


def list(project_id: str):
    client = httpx.Client()
    url = f"https://koso.app/api/projects/{project_id}/dupes"
    response = client.get(url, headers=HEADERS).json()
    print(response)


def create(project_id: str, task1: str, task2: str, similarity: float):
    client = httpx.Client()
    url = f"https://koso.app/api/projects/{project_id}/dupes"
    response = client.post(
        url,
        headers=HEADERS,
        json={
            "task1Id": task1,
            "task2Id": task2,
            "similarity": similarity,
        },
    ).json()
    print(response)


def main():
    parser = argparse.ArgumentParser(description="Manage duplicates in Koso projects")
    subparsers = parser.add_subparsers(
        dest="command", help="Available commands", required=True
    )

    list_parser = subparsers.add_parser("list", help="List duplicates for a project")
    list_parser.add_argument("project_id", help="Project ID to list duplicates for")

    create_parser = subparsers.add_parser(
        "create", help="Create a duplicate relationship"
    )
    create_parser.add_argument("project_id", help="Project ID to create duplicate in")
    create_parser.add_argument("task1", help="First task ID")
    create_parser.add_argument("task2", help="Second task ID")
    create_parser.add_argument(
        "similarity", type=float, help="Similarity score between tasks"
    )

    args = parser.parse_args()

    if args.command == "list":
        list(args.project_id)
    elif args.command == "create":
        create(args.project_id, args.task1, args.task2, args.similarity)


if __name__ == "__main__":
    main()
