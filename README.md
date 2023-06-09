# sentiment_rollup

---

sentiment_rollup retrieves every event from the events table and updates the rollups table with the sum of all sentiment by date.

---

## Setup Overview

1. Create an EC2 instance
1. Install tools on the EC2 instance
1. Setup a database
1. Install source code
1. Build
1. Create Service

### Setup

**See sentiment_collector for previous setup instructions.**


##### Create Table

```sql
CREATE TABLE rollups(
    id INT AUTO_INCREMENT,
    date DATE NOT NULL UNIQUE,
    sentiment DECIMAL(10,2) NOT NULL,
    PRIMARY KEY(id),
    INDEX (date)
);
```

#### 4. Install Source Code

```

Clone the repo on the EC2 instance.

```bash
$ git clone git@github.com:KrakenOverlord/sentiment_collector.git
```

Don't forget to copy `.env` to the EC2 instance.

```bash
$ scp -i "~/.ssh/ec2.pem" .env ec2-user@ec2-***REMOVED***.us-west-1.compute.amazonaws.com:~/sentiment_collector

```

#### 5. Build and Run Program

**Build**

```bash
./scripts/build.sh
```

**Crontab:**

#### Setup Crontab
We need to restart sentiment_collector every night at midnight because the `nostr-sdk` crate sometimes stops working. 

```bash
0 0 ** * /usr/bin/pkill -f sentiment_collector; /home/ec2-user/sentiment_collector/sentiment_collector & &>/dev/null
```

---
