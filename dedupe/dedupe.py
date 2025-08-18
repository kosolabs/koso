import argparse
import os
import httpx
import asyncio
import pandas as pd
import json
import dotenv
import openai
import numpy as np
from sklearn.metrics.pairwise import cosine_similarity
import sqlite3
from tqdm.asyncio import tqdm_asyncio
from sklearn.cluster import HDBSCAN


HEADERS = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {os.environ['KOSO_AUTH_TOKEN']}",
}


def list_dupes(project_id: str):
    client = httpx.Client()
    url = f"https://koso.app/api/projects/{project_id}/dupes"
    response = client.get(url, headers=HEADERS).json()
    print(response)


def create_dupe(project_id: str, task1: str, task2: str, similarity: float):
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
    dotenv.load_dotenv()
    db_file = "koso-tasks-embeddings.sqlite"
    file_path = "koso-dogfood-export-2025-7-5-12-2.json"

    with open(file_path, "r") as f:
        data = json.load(f)
    projectId = data.get("projectId")
    project_id = projectId

    asyncio.run(compute_embeddings(file_path, db_file))
    compute_clusters(file_path, db_file)
    dupes = compute_dupes(project_id, file_path, db_file)
    store_dupes(dupes, project_id)


async def compute_embeddings(file_path: str, db_file: str):
    client = openai.AsyncOpenAI()

    with open(file_path) as new:
        data = json.load(new)

    tasks = data["graph"]

    async def get_embedding(data):
        response = await client.embeddings.create(
            model="text-embedding-3-small", input=data, encoding_format="float"
        )
        return response.data[0].embedding

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
            ).set_index("id")
            df.to_sql("embeddings", db, if_exists="append")
            return df

    def get_processed_row_ids(db):
        try:
            return {
                row[0] for row in db.execute("Select id from embeddings").fetchall()
            }
        except sqlite3.OperationalError as e:
            print(f"Skipping delta due to error getting previously processed Id's: {e}")
        return set()

    semaphore = asyncio.Semaphore(20)

    db = sqlite3.connect(db_file, autocommit=True)

    processed_ids = get_processed_row_ids(db)
    filtered_tasks = [
        task for task in list(tasks.values()) if task["id"] not in processed_ids
    ]

    coros = []
    for task in filtered_tasks:
        coros.append(process_row(task, db, semaphore))
    await tqdm_asyncio.gather(*coros)


def compute_clusters(file_path: str, db_file: str):
    pd.set_option("display.max_colwidth", None)
    pd.options.display.max_rows = 1000
    pd.options.display.max_columns = 300

    with open(file_path, "r") as fp:
        koso_data = json.load(fp)

    # makes the data frame
    koso_data_frame = pd.DataFrame(koso_data["graph"].values())

    # defining a str that has the sqlite database file
    db = sqlite3.connect(db_file, autocommit=True)
    # autocommit=True means any changes are automatically
    # saved to databse without having to call commit manually

    df = pd.read_sql("SELECT * FROM embeddings", db)
    df["embedding"] = df["embedding"].apply(json.loads)

    # removes duplicate rows from the 'name' and 'id' columns
    # if two+ rows have the same combo of 'id' and 'name' only the 1st is kept
    df = df.drop_duplicates(subset=["id", "name"])

    sdf = df
    embeddings = list(sdf["embedding"].values)
    X = np.array(embeddings)

    # HDBSCAN groups similar points into clusters, can identify outliers/noise
    model = HDBSCAN(
        min_cluster_size=2,  # two or more to form a cluster
        store_centers="centroid",  # centroid is the center point of the cluster
    ).fit(X)

    sdf["cluster"] = model.labels_

    # drops the embedding column from the data frame
    # show the first five rows via .head()
    # sdf.drop(columns=["embedding"]).head()

    # detects outliers (-1)
    # unique cluster label,
    # and how many rows are assigned to each cluster label

    sdf["cluster"].value_counts()

    # displays the first 20 outliers found in the clustering algorithm
    sdf[sdf["cluster"] == -1].drop(columns=["embedding"]).head(20)

    sdf["cluster"].nunique()

    similarities = []
    for _, row in sdf.iterrows():
        cluster = row["cluster"]
        if cluster == -1:
            similarities.append(None)
        else:
            centroid = model.centroids_[cluster]
            similarity = cosine_similarity([centroid], [row["embedding"]])

            similarities.append(similarity[0][0])

    sdf["similarity"] = similarities

    # measures the cosine similarity betweena. data point and its cluster centroid
    # how well the point fits with its cluster
    cosine_similarity([centroid], [np.array(row["embedding"])])

    # @ means dot product function
    # this calc. the dot prodcut between cluster center and a data points embedding
    # returns a scalar that represents how "aligned" the embedding is with the centroid
    # the larger the value the more similar their directions

    centroid @ np.array(row["embedding"])

    # drops the embedding column from sdf
    # merges it with koso_data_frame and the id column
    # then shows the first 5 rows
    jdf = sdf.drop(columns=["embedding"]).merge(
        koso_data_frame[["id", "num", "status", "kind"]], on="id"
    )
    jdf.head()

    # this line returns the lowest similarity of any point to its cluster centroid
    jdf.groupby("cluster")["similarity"].min().min()

    # turns the porcessed data frame to a CSV file
    jdf.to_csv("koso-for-deduping.csv")

    # tests that returns only columns where the cluster is equal to 82
    jdf[jdf["cluster"] == 82]

    # returns a data frame of,
    # points assigned to real clusters that have HIGH simliiarity of (>0.93)
    # then sorted by cluster and then by similarity (best to worst)
    jdf[(jdf["cluster"] != -1) & (jdf["similarity"] > 0.93)].sort_values(
        by=["cluster", "similarity"], ascending=[True, False]
    )
    try:
        conn = sqlite3.connect(db_file)
        jdf.to_sql("clustered_tasks_jdf", conn, if_exists="replace", index=False)

        conn.close()

        print(
            f"DataFrame 'jdf' successfully saved to '{db_file}' as table 'clustered_tasks_jdf'."
        )

    except Exception as e:
        print(f"An error occurred while saving to SQLite: {e}")
        print(
            "Please ensure your 'jdf' DataFrame is correctly structured before saving."
        )


def compute_dupes(project_id: str, file_path: str, db_file: str):
    # load raw task data from JSON
    try:
        with open(file_path, "r") as fp:
            koso_raw_data = json.load(fp)
            # convert graph to dictionary values in DataFrame

        if koso_raw_data.get("projectId") == project_id:
            tasks_df = pd.DataFrame(koso_raw_data["graph"].values())
            print(
                f"{len(tasks_df)} tasks have been successfully loaded from the DataFrame"
            )

            tasks_df = tasks_df.dropna(subset=["id", "name"]).reset_index(drop=True)
            print(f"Filtered to {len(tasks_df)} tasks with valid IDs and names")

        else:
            print(
                f'Skipping file: Project ID "{koso_raw_data.get("project_id")}" does not match target ID "{project_id}".'
            )
            tasks_df = pd.DataFrame()

    except FileNotFoundError as e:
        print(f"Error: File not found. Please ensure it's in the same directory: {e}")
        exit()
    except json.JSONDecodeError as e:
        print(f"Error: Could not decode JSON. Check file format: {e}")
        exit()
    except KeyError as e:
        print(
            f"Error: 'graph' key not found in the JSON data. Ensure JSON structure is as expected: {e}"
        )
        exit()

    try:
        conn = sqlite3.connect(db_file)

        sdf = pd.read_sql("SELECT id, name, embedding FROM embeddings", conn)

        sdf["embedding"] = sdf["embedding"].apply(lambda x: np.array(json.loads(x)))
        print(f"Successfully loaded {len(sdf)} embeddings from 'embeddings' table.")

        jdf = pd.read_sql(
            "SELECT id, name, cluster, similarity FROM clustered_tasks_jdf", conn
        )
        print(
            f"Successfully loaded {len(jdf)} clustered tasks from 'clustered_tasks_jdf' table."
        )

        conn.close()

    except sqlite3.OperationalError as e:
        print(f"Error connecting to or reading from SQLite database '{db_file}': {e}")
        # Enhanced guidance for the user
        if "no such table" in str(e):
            print("Specifically, the table 'clustered_tasks_jdf' was not found.")
            print(
                "Please ensure you have run your 'clusteringandsimilarity.ipynb' notebook and saved the 'jdf' DataFrame to SQLite under the table name 'clustered_tasks_jdf'."
            )
            print("Example code to save jdf to SQLite:")
            print(
                "jdf.to_sql('clustered_tasks_jdf', conn, if_exists='replace', index=False)"
            )
        exit()
    except KeyError as e:
        print(f"Error: Missing expected column in loaded DataFrame: {e}")
        print(
            "Ensure your 'embeddings' table has 'id', 'name', 'embedding' and 'clustered_tasks_jdf' table has 'id', 'name', 'cluster', 'similarity'."
        )
        exit()
    except json.JSONDecodeError as e:
        print(f"Error decoding JSON from 'embedding' column in 'embeddings' table: {e}")
        print(
            "Ensure the 'embedding' column contains valid JSON strings of numerical lists."
        )
        exit()

    print("Real data loaded from SQLite. Proceeding with deduplication.")

    try:
        conn = sqlite3.connect(db_file)

        sdf = pd.read_sql("SELECT id, name, embedding FROM embeddings", conn)
        sdf["embedding"] = sdf["embedding"].apply(lambda x: np.array(json.loads(x)))
        print(
            f"Successfully loaded {len(sdf)} embeddings from 'embeddings' table in '{db_file}'."
        )

        jdf = pd.read_sql(
            "SELECT id, name, cluster, similarity FROM clustered_tasks_jdf", conn
        )
        print(
            f"Successfully loaded {len(jdf)} clustered tasks from 'clustered_tasks_jdf' table in '{db_file}'."
        )

        conn.close()

    except sqlite3.OperationalError as e:
        print(f"Error connecting to or reading from SQLite database '{db_file}': {e}")
        print(
            "ACTION REQUIRED: Please ensure the database file exists and the tables 'embeddings' and 'clustered_tasks_jdf' are present and correctly named."
        )
        print("  - 'embeddings' is typically created by your embedding.ipynb notebook.")
        print(
            "  - 'clustered_tasks_jdf' is typically created by your clusteringandsimilarity.ipynb notebook."
        )
        print(
            "  If these tables don't exist, you need to run those notebooks first to generate and save the data."
        )
        exit()
    except KeyError as e:
        print(f"Error: Missing expected column in loaded DataFrame: {e}")
        print(
            "ACTION REQUIRED: Ensure your 'embeddings' table has 'id', 'name', 'embedding' and 'clustered_tasks_jdf' table has 'id', 'name', 'cluster', 'similarity'."
        )
        exit()
    except json.JSONDecodeError as e:
        print(f"Error decoding JSON from 'embedding' column in 'embeddings' table: {e}")
        print(
            "ACTION REQUIRED: Ensure the 'embedding' column in your 'embeddings' table contains valid JSON strings of numerical lists."
        )
        exit()

    print("All necessary data loaded from SQLite. Proceeding with deduplication logic.")

    # define similarity thresholds
    AUTO_APPROVE_THRESHOLD = 0.95

    PROMPT_THRESHOLD = 0.85

    # list to store al potiental duplicate pairs that will be sent to the frontend
    potiental_duplicate_pairs = []

    # set to keep track of pairs already checked to avoid redundant comparisons
    checked_pairs = set()

    print("Identifying potiental duplicate pairs for frontend presentation...")

    # group tasks by assigend cluster

    for cluster_id, group in jdf.groupby("cluster"):
        if cluster_id == -1:
            continue  # skips outlier

        group_records = group.sort_values(by="similarity", ascending=False).to_dict(
            "records"
        )

        # compare every unique pair of tasks within the current cluster
        for i in range(len(group_records)):
            for j in range(i + 1, len(group_records)):
                task1_id = group_records[i]["id"]
                task2_id = group_records[j]["id"]
                task1_name = group_records[i]["name"]
                task2_name = group_records[j]["name"]

                # this makes (1, 2) same as (2, 1), etc
                pair_key = tuple(sorted([task1_name, task2_name]))

                if pair_key in checked_pairs:
                    continue
                checked_pairs.add(pair_key)  # mark pair as checked

                if task1_id not in sdf["id"].values or task2_id not in sdf["id"].values:
                    print(
                        f"Warning: Missing embedding for ID {task1_id} or {task2_id}. Skipping Pair."
                    )
                    continue

                # retrieves embeddings for the two tasks directly from 'sdf'
                emb1_raw = sdf.loc[sdf["id"] == task1_id, "embedding"].values[0]
                emb2_raw = sdf.loc[sdf["id"] == task2_id, "embedding"].values[0]

                if not isinstance(emb1_raw, np.ndarray) or not isinstance(
                    emb2_raw, np.ndarray
                ):
                    print(
                        f"Skipping pair due to non-ndarray embedding type for: '{task1_name}' ({task1_id}) and '{task2_name}' ({task2_id})."
                    )
                    continue

                emb1 = emb1_raw
                emb2 = emb2_raw

                # calculate cosine similarity between two emebddings
                sim = cosine_similarity(emb1.reshape(1, -1), emb2.reshape(1, -1))[0][0]

                if sim >= PROMPT_THRESHOLD:
                    potiental_duplicate_pairs.append(
                        {
                            "task1_id": task1_id,
                            "task1_name": task1_name,
                            "task2_id": task2_id,
                            "task2_name": task2_name,
                            "similarity": sim,
                            "auto_approve_candidate": sim >= AUTO_APPROVE_THRESHOLD,
                        }
                    )

    # converts list of potiental dupes into pandas DataFrame
    potiental_duplicate_pairs_df = pd.DataFrame(potiental_duplicate_pairs)

    if not potiental_duplicate_pairs_df.empty:
        potiental_duplicate_pairs_df = potiental_duplicate_pairs_df.sort_values(
            by="similarity", ascending=False
        ).reset_index(drop=True)

        print("\nSuccessfully identified potential duplicate pairs.")
        print(potiental_duplicate_pairs_df)

        # output for frontend
        output_columns = [
            "task1_id",
            "task1_name",
            "task2_id",
            "task2_name",
            "similarity",
            "auto_approve_candidate",
        ]
        csv_output_df = potiental_duplicate_pairs_df[output_columns]

        # Save the DataFrame to a CSV file
        csv_file_path = "potential_duplicates.csv"
        csv_output_df.to_csv(csv_file_path, index=False)
        print(f"\nPotential duplicate pairs saved to {csv_file_path}")
        return csv_output_df.to_dict(orient="records")
    else:
        print("\nNo potential duplicate pairs found above the PROMPT_THRESHOLD.")
        return []


def store_dupes(dupes, project_id):
    for dupe in dupes:
        create_dupe(project_id, dupe["task1_id"], dupe["task2_id"], dupe["similarity"])


if __name__ == "__main__":
    main()
