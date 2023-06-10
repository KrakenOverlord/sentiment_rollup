# sentiment_rollup

---

sentiment_rollup retrieves all events prior to todays date UTC from the events table and creates/updates the rollups table with the sum of all sentiment for each date.

---

## Setup Overview

1. Create Table
1. Install source code
1. Build
1. Cron

### Setup

**See sentiment_collector for server setup instructions.**


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

Clone the repo on the EC2 instance.

```bash
$ git clone git@github.com:KrakenOverlord/sentiment_rollup.git
```

Don't forget to copy `.env` to the EC2 instance.

```bash
$ scp -i "~/.ssh/ec2.pem" .env ec2-user@ec2-***REMOVED***.us-west-1.compute.amazonaws.com:~/sentiment_rollup

```

#### 3. Build

**Build**

```bash
./scripts/build.sh
```

#### 4. Crontab
We need to run sentiment_rollup once a day at midnight. 

```bash
0 0 * * * cd /home/ec2-user/sentiment_rollup && ./sentiment_rollup
```

---
