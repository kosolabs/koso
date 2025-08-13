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



async def clustering_similarity():
    import pandas as pd
    import numpy as np
    import sqlite3
    import json

    from sklearn.cluster import HDBSCAN
    from sklearn.metrics.pairwise import cosine_similarity

    pd.set_option('display.max_colwidth', None)
    pd.options.display.max_rows = 1000
    pd.options.display.max_columns = 300

    with open("koso-dogfood-export-2025-7-5-12-2.json", 'r') as fp: 
        koso_data = json.load(fp)

    #returns the values from the data 

    koso_data['graph'].values()

    #makes the data frame 

    koso_data_frame = pd.DataFrame(koso_data['graph'].values())
    koso_data_frame.head()

    #returns the dimension of the data frame 

    koso_data_frame.shape

    # .connect establishes a connect to the sqlite database 
    db_file = "koso-tasks-embeddings.sqlite" 
    #defining a str that has the sqlite database file
    db = sqlite3.connect(db_file, autocommit=True) 
    #autocommit=True means any changes are automatically 
    #saved to databse without having to call commit manually 

    df = pd.read_sql('SELECT * FROM embeddings', db)
    df['embedding'] = df['embedding'].apply(json.loads)


    #.head() shows the rows in a dataframe, 
    # put a number n in the () to only see the top n rows 
    df.head()

    #dimensions of data frame, from embedding data?
    df.shape

    duplicated_ids= df['id'].value_counts()[lambda s : s >1].index

    #df["id"].value_counts() ---> counts how many times each unique ID appears in 'id'
    #[lambda s : s >1] ---> a filter that keeps only the ID's that appear more than once 
    #.index ---> extracts the ID's (not the counts)


    #returns the # of unique id's in the data frame 
    df['id'].nunique()

    #returns the # of unique names in the data frame 
    df['name'].nunique()

    df['name'].value_counts()[lambda s : s > 1]
    #returns the unique names and the count of each unique name, 
    #listed in dscending order 

    #removes duplicate rows from the 'name' and 'id' columns 
    #if two+ rows have the same combo of 'id' and 'name' only the 1st is kept 

    df = df.drop_duplicates(subset=['id', 'name'])
    df.shape


    #the dimensions of the dataframe after the duplicates are removed 

    sdf = df 
    embeddings = list(sdf['embedding'].values)
    X = np.array(embeddings)

    #HDBSCAN groups similar points into clusters, can identify outliers/noise

    model = HDBSCAN(
        min_cluster_size=2, #two or more to form a cluster
        store_centers='centroid', #centroid is the center point of the cluster
    ).fit(X)

    sdf["cluster"] = model.labels_

    #drops the mebedding column from the data frame 
    #showx the first five rows via .head()
    sdf.drop(columns=['embedding']).head()

    #detects outliers (-1)
    # unique cluster label, 
    # and how many rows are assigned to each cluster label 

    sdf['cluster'].value_counts()


    #displays the first 20 outliers found in the clustering algorithm 
    sdf[sdf['cluster'] == -1].drop(columns=['embedding']).head(20)

    sdf['cluster'].nunique()

    similarities = []
    for _, row in sdf.iterrows(): 
        cluster = row['cluster']
        if cluster == -1:
            similarities.append(None)
        else: 
            centroid = model.centroids_[cluster]
            embedding = row['embedding']
            similarity = cosine_similarity([centroid], [row['embedding']])

            similarities.append(similarity[0][0])

    sdf['similarity'] = similarities

    #measures the cosine similarity betweena. data point and its cluster centroid
    #how well the point fits with its cluster
    cosine_similarity([centroid], [np.array(row['embedding'])])

    # @ means dot product function 
    #this calc. the dot prodcut between cluster center and a data points embedding 
    #returns a scalar that represents how "aligned" the embedding is with the centroid
    # the larger the value the more similar their directions

    centroid @ np.array(row['embedding'])


    #drops the embedding column from sdf
    #merges it with koso_data_frame and the id column 
    #then shows the first 5 rows 
    jdf = sdf.drop(columns=['embedding']).merge(koso_data_frame[['id', 'num', 'status', 'kind']], on='id')
    jdf.head()

    #this line returns the lowest similarity of any point to its cluster centroid 
    jdf.groupby('cluster')['similarity'].min().min()

    # turns the porcessed data frame to a CSV file
    jdf.to_csv('koso-for-deduping.csv')

    #tests that returns only columns where the cluster is equal to 82
    jdf[jdf['cluster'] == 82]

    #returns a data frame of, 
    #points assigned to real clusters that have HIGH simliiarity of (>0.93)
    #then sorted by cluster and then by similarity (best to worst)
    jdf[(jdf['cluster'] != -1) & (jdf['similarity'] > .93)].sort_values(by=['cluster', 'similarity'], ascending=[True, False])

    db_file = 'koso-tasks-embeddings.sqlite'
    try:
        conn = sqlite3.connect(db_file)
        jdf.to_sql('clustered_tasks_jdf', conn, if_exists='replace', index=False)

        conn.close()

        print(f"DataFrame 'jdf' successfully saved to '{db_file}' as table 'clustered_tasks_jdf'.")

    except Exception as e:
        print(f"An error occurred while saving to SQLite: {e}")
        print("Please ensure your 'jdf' DataFrame is correctly structured before saving.")



