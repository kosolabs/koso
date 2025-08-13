async def embeddings_notebook() :
    import asyncio
    import pandas as pd
    import json
    import dotenv
    import openai

    import sqlite3
    from tqdm.asyncio import tqdm_asyncio

    dotenv.load_dotenv()

    client = openai.AsyncOpenAI()

    response = await client.embeddings.create(model="text-embedding-3-small", input="The food was delicious and the waiter...", encoding_format="float")

    response

    with open("koso-dogfood-export-2025-7-5-12-2.json") as new: 
        embedding = json.load(new)

    tasks = embedding['graph']

    for _, task in tasks.items():
        print(task["name"])

    async def get_embedding(data): 
        response = await client.embeddings.create(model="text-embedding-3-small", input=data, encoding_format="float")
        return response.data[0].embedding

    await get_embedding("Koso will be revolutionary!")

    async def process_row(row, db, semaphore=asyncio.Semaphore(1)):
        async with semaphore:
            embedding = await get_embedding(row["name"])
            df = pd.DataFrame(
                [
                    {
                        "id": row["id"],
                        "name": row["name"],
                        "embedding": json.dumps(embedding),
                    }
                ]
            ).set_index('id')
            df.to_sql('embeddings',db, if_exists='append')
            return df

    def get_processed_row_ids(db):
        try: 
            return {row[0] for row in db.execute('Select id from embeddings').fetchall()}
        except sqlite3.OperationalError as e: 
            print(f"Skipping delta due to error getting previously processed Id's: {e}")
        return set()

    semaphore = asyncio.Semaphore(20)

    db_file = 'koso-tasks-embeddings.sqlite'
    db = sqlite3.connect(db_file, autocommit=True)

    processed_ids = get_processed_row_ids(db)
    filtered_tasks = [task for task in list(tasks.values()) if task['id'] not in processed_ids]

    coros = []

    for task in filtered_tasks: 
        coros.append(process_row(task, db, semaphore))
    await tqdm_asyncio.gather(*coros)

    tasks.items()

    for _, task in tasks.items(): 
        print(task['id'])




