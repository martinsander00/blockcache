
# Blockcache

5-minute volume tracker simulation with backend server, cache and PostgreSQL DB.

## Architecture

### Website

This website simulates the client user. It is able to see in real time the volume from a certain pool address of the last 5 minute window.

When the user clicks on a specific pool, a graph of the latest volume gets displayed.

### Server

The server takes POST requests from the client, checks if it can hit the cache first and if it has the latest volume of the desired pool available. If not, then the server performs the query itself and retrieves the volume back to the client.

If the client doesn't perform a query in 30 seconds, the server will query the db itself for all pool addresses and update the cache.

### Cache

Every 30 seconds the cache get's updated by the server. The server checks the cache if it's running and has the latest volume available.


## Set up

### Database

You need to have a PostgreSQL instance running in the background. For this do the following:

1. Install PostgreSQL (if not already installed):

If PostgreSQL is not installed on your machine, you can install it using Homebrew (for macOS):

```bash
brew install postgresql
```

Then, start the PostgreSQL service:

```bash
brew services start postgresql
```

2. Access PostgreSQL CLI:
Open the PostgreSQL command line:

```bash
psql postgres
```

3. Create a Database and User:
Create a database and user that matches your connection string.

First, create a new user:

```bash
CREATE USER myuser WITH PASSWORD 'mypassword';
```

Then, create a database:

```bash
CREATE DATABASE mydb;
```

Grant privileges on the database to the user:

```bash
GRANT ALL PRIVILEGES ON DATABASE mydb TO myuser;
```

4. Create the Table:
After connecting to the database, run your CREATE TABLE query:

```bash
psql -U myuser -d mydb
```

Once connected, create your table:

```bash
CREATE TABLE transactions (
  id SERIAL PRIMARY KEY,
  signature TEXT UNIQUE NOT NULL,
  pool_address TEXT NOT NULL,
  amount DOUBLE PRECISION NOT NULL,
  timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);
```

Add an index on:

```bash
CREATE INDEX idx_transactions_pool_timestamp
ON transactions (pool_address, timestamp DESC);
```

Adding this index benefits queries that filter by pool_address and order by timestamp DESC, reducing the need for full table scans and sorting operations.

6. Run PostgreSQL in the Background:
Make sure PostgreSQL is running in the background. You can manage the service using the following command (if you used Homebrew):

```bash
brew services start postgresql
```

### Server
Go to the server folder and run

```bash
cargo build
cargo run
```

### Client/Website

Here you have two options. The simplest one (and the one I prefer for simplicity) is to not run the website and instead use the client server. For this you would go to the client folder and run:

```bash
cargo build
cargo run
```

To use the client server, you run the following command in the terminal:

```bash
get vol <POOL ADDRESS>
```

Currently, the application supports these 3 addresses:

1. 6d4UYGAEs4Akq6py8Vb3Qv5PvMkecPLS1Z9bBCcip2R7
2. CWjGo5jkduSW5LN5rxgiQ18vGnJJEKWPCXkpJGxKSQTH
3. 7xuPLn8Bun4ZGHeD95xYLnPKReKtSe7zfVRzRJWJZVZW


If you want to use the website, which is a conceptual version of how it could look like in a dev environment, you can go to the website folder and run:

```bash
npm install
npm start
```


### Cache

This part is not required for the application to run, however it is the "cool" part of the application. To run the cache, go to the cache folder and run:

```bash
cargo build
cargo run
```

## Ready

Now you are ready to start running the <get vol> command in the client. Notice that if you are using the cache the value of the same address doesn't change, this is because the cache gets updated every 30 seconds or so for each pool address. If you shut down the cache, the value will always change.

## Website

In this website there are just two rows that are "working". The rest are dummy rows.

The price value in the rows gets updated every 10 seconds or so and turns green or red depending on the price movement. 

If you click on a row, you get an ugly looking graph that tracks the price changes of the pool address you selected.
