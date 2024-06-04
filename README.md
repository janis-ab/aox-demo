# Demo code

This code is only for educational purposes.

## Task Description:
Your task is to create a Rust application that fulfills the following
requirements:

1. Utilize a multi-threaded approach to continuously retrieve real-time data
from an external API. External API can be of your choice that doesn't require
apikey [1]. If real-time data is not available, you may simulate and process
past data as if it were current.
2. Perform meaningful metrics calculations to display output in terminal,
updating every second.
2. Implement an asynchronous task scheduler using tokio runtime that periodically stores your calculated data after specific time-intervals.
3. Store data in a SQL or NoSQL like database or optionally file.
4. Include a README file with description about your project.

Optional Tasks:
1. Dockerize your project for easy setup and execution.
2. Write unit tests to ensure the correctness of your code.

Instructions:
1. Fork this repository into your GitHub account.
2. Implement the required functionality as described above.
3. Ensure your code follows best practices.



## Task solution:
1. &#9745; Multi-threaded approach is used, since we spawn 4 async functions
that are managed by Tokio multi threaded runtime.
1. &#9745; Real-time data is taken from https://docs.coincap.io/#2a87f3d4-f61f-42d3-97e0-3a9afa41c73b as provided in [1]. In real life we might
want to select broader data scope, but this i think is fine for demo example.
Many links to APIs are not valid anymore (repo has some outdated info).

2. &#9745; Program selects multiple tick information 1 per second and aggregates
it into open-high-low-close value struct. Data is collected asynchronously
within separate thread, real time calculations are printed on screen once per
second, data is stored in Postgresql database in configured time intervals,
stored data is 1 minute OHLC price for Bitcoin.

2. &#9745; Asynchronous info update 1 per second in terminal console
implemented; regardless incomming data rate.

3. &#9745; Data is stored in Postgresql demo database, table public.ohlc. Other storage
backends could be implemented in similar fashion. There is a special trait
Storage that can be implemented to introduce new configurable storage backend.

4. &#9745; Project [README](service_demo/README) file is available.

Optional Tasks:
1. &#9745; Docker image building implemented (tested on Debian GNU/Linux 12 (bookworm)),
2. &#9745; Some unit tests written.
3. &#9745; Hopefully code follows best practices.

# Notes
Errors are printed to STDERR. Depending on ENV requirements, special logger
could be used instead. Calculated information is printed to STDOUT.

OHLC calculation is done based on Unix timestamp minutes, it does not take into
account leap seconds.

Structs that are sent over channels have public properties. This is for ease of
use. We could abstract them away with getter and setter methods.

Since this process is focused on working with real time data, when storage
backend, or queue can not keep up with incomming data flow, unhandled data is
dropped. This approach is taken to prioritize latest data availability at the
cost of data loss. For data analysis, historical data could be loaded from
alternative API endpoint, but realtime processes must move on as fast as they
can and stale data can hinder quality of decissions made by algorithm.

In our docker-compose file we set that service is not rebooted upon exit. This
is convenient for testing and demonstration purposes; in production we would
use different settings.

# To build demo
Change into directory where this repository was downloaded and run:
```sh
cp .env.example .env
cp service_demo/.env.example service_demo/.env
docker compose build demo
```


# To run demo
This allows to see terminal output. But sometimes database container does not
stop on Ctrl+C.
```sh
docker compose up demo
```

This is to start daemonized docker container. It is not possible to see terminal
output directly, but container shuts down cleanly with `docker compose down`.
```sh
docker compose up -d demo
```


# To stop container
```sh
docker compose down
```


# To select collected data from DB
Reminder: new rows are inserted once per minute. Data is collected in 800ms
intervals, and terminal is updated once per second. So new data in DB should
be available aprox. after 1 minute of execution.

DB has no persistent storage, thus on each reboot data is lost.
```sh
docker compose exec database psql -h aox-database -U demouser -c 'SELECT * FROM ohlc LIMIT 10' demo
```


# Short docker install instructions on Debian
Short instruction of how to install docker.

Do NOT run this in production without adjustment, since it might overwrite
existing configuration without user confirmation.

```sh
apt-get update
apt-get install -y curl ca-certificates gnupg lsb-release

# Download Docker's official GPG key
mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/debian/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg


# Configure apt repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/debian $(lsb_release -cs) stable" | tee /etc/apt/sources.list.d/docker.list > /dev/null 


# Update repository to include new info
apt-get update

apt-get install -y docker-ce docker-ce-cli containerd.io docker-compose-plugin
docker run hello-world
```



[1] - https://github.com/public-apis/public-apis


