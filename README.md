# sentiment_collector

---

sentiment_collector subscribes to nostr relays to receive all TextNote events from each one. If the content contains the word `bitcoin`, it quantifies the sentiment using one of the user configurable sentiment analysis tools:

1. vader_sentiment crate
1. OpenAI API
1. Bard API

It then records the event ID and it's sentiment in one of the user configurable databases:

1. MariaDB
1. DynamoDB

Sentiment values range from [-1, 1] and event ID's are guaranteed to be unique.

---

## Setup Overview

1. Create an EC2 instance
1. Install tools on the EC2 instance
1. Setup a database
1. Install source code
1. Build
1. Create Service

### Setup

#### 1. Create an EC2 Instance

- Use Amazon Linux 2023
- Use an pre-existing keypair if you have one, or create a new keypair.
- Open Port 3306 on the EC2 Instance if using the MariaDB database

 **Connect to the EC2 Instance**

```bash
ssh -i "~/.ssh/ec2.pem" ec2-user@ec2-***REMOVED***.us-west-1.compute.amazonaws.com
````

#### 2. Install Tools on the EC2 Instance


```bash
$ sudo dnf update -y
$ sudo yum groupinstall "Development Tools"
$ install rustup `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
$ sudo dnf install git-all
```

#### 3. Setup a Database

*You can uses either MariaDB running locally, or DynamoDB running on AWS.*

##### DynamoDB
- table name `sentiments`
- `pk` String
- `sk` Number

#####  MariaDB

```bash
$ sudo dnf install mariadb105-server`
$ sudo systemctl start mariadb
$ sudo mysql_secure_installation
$ sudo systemctl enable mariadb
```

*To Stop MariaDB `$ sudo systemctl stop mariadb`*

**Connect to MariaDB**
`$ mysql -h localhost -u root -p`

##### Configure Access for Remote User

```sql
CREATE USER 'user'@'localhost' IDENTIFIED BY 'pwd';

CREATE USER 'user'@'%' IDENTIFIED BY 'pwd';

GRANT ALL PRIVILEGES ON *.* to user@localhost IDENTIFIED BY 'pwd' WITH GRANT OPTION;

GRANT ALL PRIVILEGES ON *.* to user@'%' IDENTIFIED BY 'pwd' WITH GRANT OPTION;

FLUSH PRIVILEGES;

EXIT
```

##### Create Database and Tables

```sql
DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;

CREATE DATABASE sentiment;

CREATE TABLE events(
    id INT AUTO_INCREMENT,
    event_id VARCHAR(256) NOT NULL UNIQUE,
    sentiment DECIMAL(2,2) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id),
    INDEX (created_at)
);
```

**Monitor Maria Database Size**

```sql
select table_schema, sum((data_length+index_length)/1024/1024) AS MB from information_schema.tables group by 1;`
```

#### 4. Install Source Code

Move your GitHub keys to the EC2 instance so that we can pull the sentiment_collector repo.

```bash
$ scp -i "~/.ssh/ec2.pem" ~/.ssh/id_rsa* ec2-user@ec2-***REMOVED***.us-west-1.compute.amazonaws.com:~/.ssh
```

Now we can clone the repo on the EC2 instance.

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

#### 6. Create Service

Create a unit file named `sentiment_collector.service` and place it into `/etc/systemd/system`: 

```bash
[Unit]
Description=Service that keeps sentiment_collector running.
After=network.target

[Install]
WantedBy=multi-user.target

[Service]
Type=simple
ExecStart=/home/ec2-user/sentiment_collector/sentiment_collector
WorkingDirectory=/home/ec2-user/sentiment_collector
Restart=always
RestartSec=1
StandardOutput=syslog
StandardError=syslog
SyslogIdentifier=%n
```

```
$ sudo systemctl daemon-reload
$ sudo systemctl enable sentiment_collector.service
$ sudo systemctl start sentiment_collector.service
```

**Check Status**
`$ sudo systemctl status sentiment_collector.service`

**Logs**
`$ journalctl -f -u sentiment_collector.service`

**Or use Screen and Crontab:**

**Start Program**

```bash
screen
./sentiment_collector
Ctrl+a d
```

**Stop Program**

```bash
$ screen -r
$ pkill sentiment_collector
$ exit
```

#### Setup Crontab
We need to restart sentiment_collector every night at midnight because the `nostr-sdk` crate sometimes stops working. 

```bash
0 0 ** * /usr/bin/pkill -f sentiment_collector; /home/ec2-user/sentiment_collector/sentiment_collector & &>/dev/null
```

---
