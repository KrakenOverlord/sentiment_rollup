# sentiment_rollup

---

*Background: The sentiment_collector is constantly creating new `event` records in the `sentiment` database for each Nostr text note event that contains relevant bitcoin content. The event records contain a quantized sentiment value for the bitcoin content.*

**sentiment_rollup** creates new `rollup` records with cumulative sentiment totals for each date along with the current bitcoin price. It then deletes the processed event records.

## Setup Overview

1. Create Table
1. Install Source Code
1. Build
1. Cron

### Setup

*See sentiment_collector for server and database setup instructions.*


#### 1. Create Table

```sql
CREATE TABLE rollups(
    id INT AUTO_INCREMENT,
    date DATE NOT NULL UNIQUE,
    sentiment FLOAT(7,2) NOT NULL,
    PRIMARY KEY(id),
    INDEX (date)
);
```

#### 2. Install Source Code

Clone the sentiment_rollup repo on the server.

```bash
$ git clone git@github.com:KrakenOverlord/sentiment_rollup.git
```

Then create a `.env` file from the `.env.sample` file.

#### 3. Build


```bash
$ ./scripts/build.sh
```

#### 4. Cron
Run sentiment_rollup every hour.

```bash
0 * * * * cd [path]/sentiment_rollup && ./sentiment_rollup
```
