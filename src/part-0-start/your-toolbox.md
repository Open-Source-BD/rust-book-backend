# Your toolbox

> **Beginner** · Part 0 · Before you start

## By the end of this lesson

- Rust, Docker and an HTTP client are installed and checked.
- A Postgres database is running in Docker on your computer.
- You can connect to it and run your first SQL query.

## What & why

Install and check every tool the book uses, and start your own Postgres database.

A carpenter doesn't start a job by hunting for a hammer. They open their toolbox, check every tool
is there and working, and only then start building. This lesson is your toolbox check. Each tool
has one job:

| Tool | Its job | When you install it |
|---|---|---|
| **Rust + Cargo** | Build and run your Rust code. | You checked it in [How to use this book](how-to-use-this-book.md). |
| **Docker** | Runs programs inside sealed-off boxes, so you can use Postgres without installing it into your system. | Now. |
| **PostgreSQL** ("Postgres") | The [database](../glossary.md#database): it stores your app's data safely, even when your program stops. | Now, *inside* Docker. |
| **`psql`** | A command-line program for typing [SQL](../glossary.md#sql) (the language databases understand) straight to Postgres. | Now, or use it through Docker with nothing to install. |
| **curl** | An HTTP [client](../glossary.md#client) (a program that sends requests): it sends [HTTP](../glossary.md#http) requests (the messages web programs send each other) from your terminal, so you can test your backend without a website. | Now (it's usually already there). |
| **Bruno** | Optional: a desktop app that does the same job as curl, with buttons and forms instead of typed commands. | Only if you prefer clicking to typing. |
| **`sea-orm-cli`** | A helper for SeaORM, the database library this book uses. | **Later**, in Part A3. You don't need it yet. |

The big idea of this lesson is Docker. Installing a database the traditional way spreads files all
over your computer, starts it in the background every time you log in, and makes it hard to remove
cleanly. Docker keeps Postgres in a box instead: one command starts it, one command stops it, and
if anything goes wrong you throw the box away and get a fresh one.

## The idea, slowly

### Step 1: check Docker

[**Docker**](../glossary.md#docker) is a tool that runs programs inside
[**containers**](../glossary.md#container): small, sealed-off boxes that hold a program together with
everything it needs, kept separate from the rest of your computer. If you don't have Docker yet,
install **Docker Desktop** (Windows, macOS, Linux) from the link in *Go deeper*. On macOS,
**OrbStack** is a lighter alternative that understands exactly the same commands. Start it once
after installing, then check it from a terminal:

```bash
docker --version
docker compose version
```

### Line by line

`docker --version`
- **What:** prints the version of the Docker command-line tool.
- **Why:** every Postgres command in this book goes through `docker`, so it must be installed and
  on your terminal's path (the list of folders your terminal searches for programs).
- **How:** `docker` is the tool; `--version` is an option (an extra instruction after the command
  name) meaning "only tell me your version".
- **Remove it and…** you'd first find out Docker is missing when `docker compose up` fails.

`docker compose version`
- **What:** prints the version of **Docker Compose**, the part of Docker that reads a settings file
  describing one or more containers and starts them all with one command.
- **Why:** this book describes its database in a Compose file (Step 2), so we need Compose, not
  only Docker.
- **How:** `compose` is a *subcommand* of `docker` (a command inside a command), and `version` is
  the subcommand of `compose` that prints its version. Note: it's written without dashes
  (`version`, not `--version`) and with a space in `docker compose`.
- **Remove it and…** an old Docker without Compose would only fail later, in Step 3.

### Run it

```bash
docker --version
docker compose version
```

```text
Docker version 29.4.0, build 9d7ad9f
Docker Compose version v5.1.2
```

- ✅ If you see two version lines, you're right. Any recent version is fine; your numbers will
  differ.
- ❌ If you see `command not found: docker`, Docker isn't installed (or the terminal was opened
  before you installed it). Install Docker Desktop or OrbStack, start it, then open a **new**
  terminal.
- ❌ If `docker compose version` says `compose` is not a docker command, your Docker is very old.
  Update Docker Desktop and try again.

### Step 2: the file that describes our database

Instead of typing a long command with every setting each time, the book keeps all the database
settings in one file, `docker-compose.yml`, at the top of the book's repository. Create a folder for
your work (the book's own folder is called `rust-book-backend`), and save this file in it as
`docker-compose.yml`. It's written in **YAML**, a plain-text settings format where indentation (the
spaces at the start of a line) shows what belongs inside what, and a line starting with `#` is a
comment for humans that Docker ignores.

```yaml
{{#include ../../docker-compose.yml}}
```

### Line by line

`# Postgres for the whole book. Host port 5433 (not 5432) so it never clashes`
`# with a Postgres you may already have installed on your computer.`
- **What:** a two-line comment.
- **Why:** it tells the next person who opens the file the one surprising choice in it (port 5433),
  and why. The `ports:` entry below explains it in full.
- **How:** Docker skips every line that starts with `#`.
- **Remove it and…** nothing breaks, but the odd-looking 5433 loses its explanation.

`services:`
- **What:** starts the list of containers this file describes. Compose calls each one a
  *service*.
- **Why:** Compose needs to know which containers to create. A bigger project might list a
  database, a cache and a web server here; we need only the database.
- **How:** everything indented under `services:` belongs to it.
- **Remove it and…** Compose has nothing to start and refuses the file.

`db:`
- **What:** names our one service `db`.
- **Why:** you use this name in commands, like `docker compose logs db` or
  `docker compose exec db …`.
- **How:** the name is up to us; everything indented under it is that service's settings.
- **Remove it and…** the settings below it have no service to belong to, and the file is invalid.

`image: postgres:18`
- **What:** says which **image** to build the container from. An image is a ready-made,
  read-only template of a program and everything it needs; a container is a running copy of it.
- **Why:** `postgres` is the official Postgres image, and `:18` (the *tag*) pins it to Postgres
  version 18, so everyone reading this book gets the same database.
- **How:** the first time you start the service, Docker downloads the image from Docker Hub (a
  public library of images) and keeps it for next time.
- **Remove it and…** Compose doesn't know what to run and refuses to start. Remove only `:18` and
  you get "the newest version", which may behave differently from the book.

`environment:`
- **What:** starts a list of [**environment variables**](../glossary.md#environment-variable),
  named settings handed to the program when it starts.
- **Why:** the Postgres image reads these to set itself up.
- **How:** each indented `NAME: value` line becomes one variable inside the container.
- **Remove it and…** the image refuses to start, because it requires at least a password.

`POSTGRES_USER: postgres` · `POSTGRES_PASSWORD: postgres` · `POSTGRES_DB: rbh`
- **What:** the login details: the user name (`postgres`), that user's password (`postgres`), and
  the name of an empty database to create for us (`rbh`, short for **R**ust **B**ackend for
  **H**umans).
- **Why:** you'll need all three to connect, both from `psql` in this lesson and from Rust later.
- **How:** Postgres reads them **only the very first time** it starts with an empty data folder,
  and uses them to create the user and database. After that they're stored in the data, and
  changing them here has no effect on the existing data.
- **Remove it and…** without `POSTGRES_PASSWORD` the container stops with an error. Without the
  other two you'd get the defaults (user `postgres`, database `postgres`), and the connection URL
  later in this lesson would no longer match.

A password of `postgres` is fine for a database that only runs on your own laptop and holds
practice data. **Never** use it for a real server on the internet; Part B shows how to handle real
secrets.

`ports:` / `- "5433:5432"`
- **What:** connects a [**port**](../glossary.md#port) on your computer to a port inside the
  container. A port is a numbered door on a computer; one program listens behind each door.
- **Why:** Postgres listens on port 5432 *inside* its box. Your programs outside the box need a
  door on your own computer that leads to it. The format is **`host:container`**: the left number
  is the door on your computer (the *host*), the right number is the door inside the container.
  **Why 5433 and not 5432?** 5432 is Postgres's usual port, so if you already have Postgres
  installed (many people do, often without remembering), 5432 is taken. Using 5433 on your side
  means the book's database never fights with yours.
- **How:** Docker forwards everything that arrives at port 5433 on your computer to port 5432 in
  the container. The quotes around `"5433:5432"` stop YAML from misreading the numbers-and-colon
  as something else.
- **Remove it and…** Postgres still runs, but nothing outside the container can reach it: `psql`
  and your Rust code get `Connection refused`.

`volumes:` / `# Postgres 18 images keep data under /var/lib/postgresql/18/..., so mount the parent.` / `- pgdata:/var/lib/postgresql`
- **What:** keeps the database's files in a **volume**, a storage area Docker manages *outside*
  the container, named `pgdata`.
- **Why:** containers are meant to be thrown away and recreated. A volume means your data survives
  `docker compose down`, restarts and computer reboots: the next container picks up the same
  volume. The comment explains the path: Postgres 18 stores its data in a version-numbered folder
  under `/var/lib/postgresql` (for example `/var/lib/postgresql/18/docker`), so we attach the volume
  to that parent folder and everything below it is covered.
- **How:** `pgdata:/var/lib/postgresql` means "attach the volume named `pgdata` at the folder
  `/var/lib/postgresql` inside the container". The `-` starts an item in a list.
- **Remove it and…** your data lives in a nameless volume tied to that one container. After
  `docker compose down` and `up`, you'd get a new, empty database and your tables would seem to
  have vanished.

`healthcheck:`
- **What:** tells Docker how to test whether Postgres is ready to answer, not only started.
- **Why:** a container starts in a second, but Postgres needs a few more seconds to get ready.
  In Step 3 we'll ask Compose to `--wait` until the service is *healthy*; this block is what
  "healthy" means.
- **How:** Docker runs the `test` command every `interval`, and marks the container healthy as
  soon as the command succeeds.
- **Remove it and…** `--wait` has nothing to wait for, returns immediately, and your first query
  can fail because Postgres isn't ready yet.

`# Check over TCP (127.0.0.1), not the Unix socket: …` (three comment lines)
- **What:** explains why the check below includes `-h 127.0.0.1`.
- **Why:** it records a real trap. On its **very first start**, the Postgres image runs a
  *temporary* Postgres to create your user and database, then shuts it down and starts the real
  one. The temporary one only accepts connections through the **Unix socket**, a special file that
  programs inside the same machine use to talk to each other without any network. A check that
  used the socket would say "ready!" while the temporary server was up; `--wait` would return, and
  your first query would hit the moment of the restart and fail with
  `server closed the connection unexpectedly`.
- **How:** **TCP** is the ordinary network way programs connect (the same way your computer's
  programs will reach Postgres through port 5433). The temporary server doesn't listen on TCP,
  so a TCP check can only succeed once the real server is up.
- **Remove it and…** (the comment) nothing breaks, but a future reader might "tidy up" the `-h`
  option and bring the trap back.

`test: ["CMD-SHELL", "pg_isready -h 127.0.0.1 -U postgres -d rbh"]`
- **What:** the health test itself.
- **Why:** `pg_isready` is a small program that comes with Postgres and answers one question:
  "is the server accepting connections?" It succeeds only when the answer is yes.
- **How:** `"CMD-SHELL"` means "run the next piece of text as a command inside the container, the
  same way a terminal would".
  `-h 127.0.0.1` is the host to check: `127.0.0.1` is the numeric address that always means "this
  same machine" (the numeric form of [localhost](../glossary.md#localhost)), which forces a TCP
  connection instead of the socket (see the comment above). `-U postgres` is the user and
  `-d rbh` the database to check.
- **Remove it and…** a health check without a `test` has nothing to run, so Docker can never mark
  the container healthy.

`interval: 2s` · `timeout: 3s` · `retries: 15`
- **What:** how often to run the test (every 2 seconds), how long one test may take before it
  counts as failed (3 seconds), and how many failures in a row before the container is marked
  unhealthy (15).
- **Why:** checking every 2 seconds means `--wait` returns soon after Postgres is ready; 15 retries
  give it about half a minute, plenty even on a slow laptop.
- **How:** Docker keeps a count of failures and resets it after a success.
- **Remove it and…** Docker uses its defaults (a check every 30 seconds), so `--wait` could sit
  idle for half a minute even when Postgres is ready.

`volumes:` / `pgdata:` (the last two lines, not indented under `db`)
- **What:** declares the named volume `pgdata` that the `db` service uses.
- **Why:** Compose requires every named volume a service uses to be declared at the top level.
- **How:** the empty entry means "a normal volume with default settings". Compose names it after
  your folder, for example `rust-book-backend_pgdata`.
- **Remove it and…** Compose refuses the file, saying the service refers to an undefined volume.

### Step 3: start the database

From the folder that contains `docker-compose.yml`, start Postgres and then ask Compose what's
running:

```bash
docker compose up -d --wait
docker compose ps
```

### Line by line

`docker compose up -d --wait`
- **What:** creates and starts every service in `docker-compose.yml`, here the `db` service.
- **Why:** this is the one command you'll use to turn the book's database on.
- **How:** `up` reads `docker-compose.yml` from the current folder; `-d` (*detached*) runs the
  container in the background and gives you your terminal back; `--wait` doesn't return until the
  health check from Step 2 passes.
- **Remove it and…** without `-d`, Postgres's log fills your terminal and stopping it with
  `Ctrl+C` stops the database. Without `--wait`, the command returns before Postgres is ready.

`docker compose ps`
- **What:** lists the containers this Compose file started, with their status.
- **Why:** it's the quickest way to see whether the database is up, healthy, and on which port.
- **How:** `ps` is short for *process status*, an old Unix name for "what's running?".
- **Remove it and…** nothing breaks; you'd lose a quick look at what's going on.

### Run it

```bash
docker compose up -d --wait
docker compose ps
```

```text
 Network rust-book-backend_default Creating 
 Network rust-book-backend_default Created 
 Volume rust-book-backend_pgdata Creating 
 Volume rust-book-backend_pgdata Created 
 Container rust-book-backend-db-1 Creating 
 Container rust-book-backend-db-1 Created 
 Container rust-book-backend-db-1 Starting 
 Container rust-book-backend-db-1 Started 
 Container rust-book-backend-db-1 Waiting 
 Container rust-book-backend-db-1 Healthy 
NAME                     IMAGE         COMMAND                  SERVICE   CREATED         STATUS                   PORTS
rust-book-backend-db-1   postgres:18   "docker-entrypoint.s…"   db        3 seconds ago   Up 2 seconds (healthy)   0.0.0.0:5433->5432/tcp, [::]:5433->5432/tcp
```

Compose named everything after your folder (`rust-book-backend`), so your names follow your folder's
name. The `Network` lines are a small private network Compose creates so its containers can talk to
each other; you don't need to do anything with it. The very first time, you'll also see Docker download the `postgres:18` image before this;
that happens once.

- ✅ If the last `up` line says `Healthy` and `ps` shows `(healthy)` and `0.0.0.0:5433->5432/tcp`,
  you're right: Postgres is running and reachable on port 5433 of your computer.
- ❌ If you see `port is already allocated`, something else is using port 5433. See
  **Common mistakes**.
- ❌ If you see `failed to connect to the docker API` (or `Cannot connect to the Docker daemon`),
  Docker isn't running. Start Docker Desktop or OrbStack and try again.

### Step 4: write down where the database is

Your Rust code will need to know where the database lives. Instead of writing that into the code,
we keep it in a file called `.env` (a plain-text file of `NAME=value` settings for your computer
only). The book ships an example you copy:

```bash
cp .env.example .env
```

This is `.env.example` (save it next to `docker-compose.yml` if you're building your own folder):

```bash
{{#include ../../.env.example}}
```

### Line by line

`cp .env.example .env`
- **What:** copies `.env.example` to a new file called `.env`.
- **Why:** `.env.example` is shared and safe to publish; `.env` is *your* copy, which you may
  change (for example if you move to a different port). The book's `.gitignore` file tells **git** (the tool that saves
  the history of your code) never to save `.env`, so your personal settings and any real passwords stay on your computer.
- **How:** `cp` is the copy command: `cp FROM TO`. (On Windows PowerShell, `cp` works too.)
- **Remove it and…** later in the book, your Rust programs won't find `DATABASE_URL` and stop with
  an error saying it isn't set.

`# Copy to .env (cp .env.example .env). Never commit .env.`
- **What:** a comment reminding you what to do with this file.
- **Why:** anyone opening the file sees at once that it's a template, not the live settings.
- **How:** lines starting with `#` are ignored.
- **Remove it and…** nothing breaks; the reminder is gone.

`DATABASE_URL=postgres://postgres:postgres@localhost:5433/rbh`
- **What:** one environment variable, `DATABASE_URL`, holding the database's full address as a
  **URL** (the same kind of address as a web link).
- **Why:** one line tells any program how to find and log in to the database. Your Rust code will
  read this variable instead of hard-coding the address.
- **How:** the URL has five parts after the `postgres://` at the front:

```text
postgres://postgres:postgres@localhost:5433/rbh
└──┬───┘   └──┬───┘ └──┬───┘ └───┬───┘ └┬─┘ └┬┘
   │          │        │         │      │    └──  DATABASE: rbh
   │          │        │         │      └───────  PORT: 5433
   │          │        │         └──────────────  HOST: localhost
   │          │        └────────────────────────  PASSWORD: postgres
   │          └─────────────────────────────────  USER: postgres
   └────────────────────────────────────────────  scheme: postgres (which kind of database)
```

  The general shape is `postgres://USER:PASSWORD@HOST:PORT/DATABASE`. Every part matches something
  from Step 2: USER, PASSWORD and DATABASE are the three `POSTGRES_…` settings, PORT is the *left*
  number of `"5433:5432"`, and HOST is `localhost`, the name that always means "this computer".
- **Remove it and…** nothing in Rust knows where the database is. Change one part (say, the port)
  without changing the Compose file to match, and connections fail.

### Run it

```bash
cp .env.example .env
cat .env
```

```text
# Copy to .env (cp .env.example .env). Never commit .env.
DATABASE_URL=postgres://postgres:postgres@localhost:5433/rbh
```

`cat` prints a file to the terminal, so you can check the copy.

- ✅ If you see the `DATABASE_URL` line, you're right.
- ❌ If you see `No such file or directory`, you're in the wrong folder. Move (`cd`) into the folder
  that contains `.env.example` and try again.

### Step 5: talk to Postgres with `psql`

Now the real test: connect to the database and ask it a question in SQL. If you have `psql`
installed (it comes with any Postgres install), you can pass it the same URL:

```bash
psql postgres://postgres:postgres@localhost:5433/rbh -c 'select version();'
```

No `psql` on your computer? You don't need to install it. The Postgres container has its own
`psql` inside, and you can run that one instead:

```bash
docker compose exec db psql -U postgres -d rbh -c 'select version();'
```

### Line by line

`psql postgres://postgres:postgres@localhost:5433/rbh`
- **What:** starts `psql` and connects it to the database at that URL.
- **Why:** it proves the whole chain works: Docker is running, port 5433 leads into the container,
  and the user, password and database are right.
- **How:** `psql` understands the same `postgres://` URL that's in your `.env`.
- **Remove it and…** (the URL) `psql` tries a Postgres on the usual port 5432 with your computer's
  user name, which is not the book's database.

`-c 'select version();'`
- **What:** runs one SQL command, prints the answer, and exits.
- **Why:** `select version();` is the smallest useful question you can ask Postgres: "which version
  are you?". Getting an answer proves you're really connected.
- **How:** `-c` means *command*. `select` asks Postgres to give back a value; `version()` is a
  built-in function that returns the version text; the `;` ends the SQL statement. The single
  quotes keep your terminal from treating `(`, `)` and `;` as its own symbols.
- **Remove it and…** without `-c …`, `psql` opens an interactive prompt (`rbh=#`) where you type
  SQL yourself; type `\q` to leave it.

`docker compose exec db psql -U postgres -d rbh -c 'select version();'`
- **What:** runs the `psql` that lives *inside* the `db` container.
- **Why:** it works on any computer with Docker, even without `psql` installed.
- **How:** `exec db` means "run this command inside the running `db` service". Since `psql` is
  already inside the box with Postgres, it doesn't need the host or port: `-U postgres` gives the
  user and `-d rbh` the database.
- **Remove it and…** you'd need your own `psql` installed to run any SQL.

### Run it

```bash
psql postgres://postgres:postgres@localhost:5433/rbh -c 'select version();'
```

```text
                                                         version                                                          
--------------------------------------------------------------------------------------------------------------------------
 PostgreSQL 18.6 (Debian 18.6-1.pgdg13+2) on aarch64-unknown-linux-gnu, compiled by gcc (Debian 14.2.0-19) 14.2.0, 64-bit
(1 row)
```

The `docker compose exec` form prints the exact same answer. `psql` shows every answer as a table:
the column name (`version`), a line of dashes, the rows, and how many rows there were. The `18.6`
is the exact Postgres version; your last digit may be different, and `aarch64` becomes `x86_64` on
an Intel or AMD computer.

- ✅ If you see `PostgreSQL 18.` and `(1 row)`, you're right: you've run your first SQL query.
- ❌ If you see `Connection refused` and `Is the server running on that host and accepting TCP/IP
  connections?`, the database isn't running. Run `docker compose up -d --wait` first.
- ❌ If you see `command not found: psql`, use the `docker compose exec` form above.
- ❌ If you see `password authentication failed`, see **Common mistakes**.

### Step 6: check curl

From Part A2 on, you'll send requests to your own backend with **curl**, a small program that sends
an HTTP [request](../glossary.md#request) and prints the [response](../glossary.md#response). It
comes with macOS, Windows 10 and later, and almost every Linux. Check it:

```bash
curl --version
```

### Line by line

`curl --version`
- **What:** prints curl's version and what it supports.
- **Why:** every "try your API" step in the book uses curl, so it must be there.
- **How:** the same `--version` option as before.
- **Remove it and…** you'd find out curl is missing only when your first API test fails.

### Run it

```bash
curl --version
```

```text
curl 8.7.1 (x86_64-apple-darwin25.0) libcurl/8.7.1 (SecureTransport) LibreSSL/3.3.6 zlib/1.2.12 nghttp2/1.68.1
Release-Date: 2024-03-27
Protocols: dict file ftp ftps gopher gophers http https imap imaps ipfs ipns ldap ldaps mqtt pop3 pop3s rtsp smb smbs smtp smtps telnet tftp
Features: alt-svc AsynchDNS GSS-API HSTS HTTP2 HTTPS-proxy IPv6 Kerberos Largefile libz MultiSSL NTLM SPNEGO SSL threadsafe UnixSockets
```

Only the first line matters: it says curl is installed. The rest lists what it can do; `http` in
the `Protocols` line is the one this book uses.

- ✅ If the first line starts with `curl 7.` or `curl 8.`, you're right.
- ❌ If you see `command not found: curl`, install it with your system's package manager (for
  example `sudo apt install curl` on Ubuntu or Debian).

### Step 7: stop the database (and the one command to be careful with)

When you're done for the day, stop the database. There are two ways, and the difference matters:

```bash
docker compose down
docker compose down -v
```

### Line by line

`docker compose down`
- **What:** stops and removes the container and its network. **Your data stays** in the `pgdata`
  volume.
- **Why:** it frees your computer's memory and port 5433 when you're not working. Next time,
  `docker compose up -d --wait` brings back the same database with everything in it.
- **How:** `down` is the opposite of `up`. It removes containers, not volumes.
- **Remove it and…** Postgres keeps running in the background until you stop Docker. That's
  harmless, but it uses memory.

`docker compose down -v`
- **What:** does everything `down` does, **and deletes the `pgdata` volume. Every table and every
  row you created is gone for good.**
- **Why:** sometimes you *want* a completely fresh database, for example to fix the
  `password authentication failed` problem in **Common mistakes**, or to start a Part over from a
  clean slate.
- **How:** `-v` is short for *volumes*: "also remove the volumes this file declared".
- **Remove it and…** (the `-v`) your data is kept. When in doubt, leave `-v` off.

### Run it

Run only the first command for now (the second one would delete your data):

```bash
docker compose down
```

```text
 Container rust-book-backend-db-1 Stopping 
 Container rust-book-backend-db-1 Stopped 
 Container rust-book-backend-db-1 Removing 
 Container rust-book-backend-db-1 Removed 
 Network rust-book-backend_default Removing 
 Network rust-book-backend_default Removed 
```

Notice what's *not* in the list: the volume. For comparison, this is what `down -v` prints; the two
extra `Volume … Removed` lines are your data being deleted:

```text
 Container rust-book-backend-db-1 Stopping 
 Container rust-book-backend-db-1 Stopped 
 Container rust-book-backend-db-1 Removing 
 Container rust-book-backend-db-1 Removed 
 Volume rust-book-backend_pgdata Removing 
 Network rust-book-backend_default Removing 
 Volume rust-book-backend_pgdata Removed 
 Network rust-book-backend_default Removed 
```

- ✅ If `down` shows `Removed` lines and no `Volume` line, you're right: stopped, data kept.
- ❌ If you see `no configuration file provided: not found`, you're in the wrong folder. `cd` into
  the folder with `docker-compose.yml`.

## You might be wondering…

**"Why Docker instead of installing Postgres normally?"**
Three reasons. Everyone gets the *same* Postgres 18, whatever their computer, so the outputs in this
book match yours. Nothing is installed into your system, and nothing starts on its own when you log
in. And when something goes wrong, `docker compose down -v` and `up` give you a brand-new database
in seconds. You'll also use the same Docker skills in Part B to ship your finished backend.

**"What is a port, really?"**
Think of your computer as an apartment building with one street address. A
[port](../glossary.md#port) is an apartment number: 65,535 doors, each with at most one program
listening behind it. Postgres usually lives behind door 5432; the book's copy lives behind 5433 so
the two never collide. Later your own Rust [server](../glossary.md#server) will live behind door
3000.

**"Is the password `postgres` safe?"**
On your own laptop, for practice data, yes. The database is only reachable from your computer's
network, and it holds nothing secret. On a real server on the internet, **never**: anyone who
guesses `postgres` could read or delete everything. Part B shows how to keep real passwords out of
your files.

**"Where does my data actually live?"**
In the Docker volume `rust-book-backend_pgdata` (Compose adds your folder's name to the front of
`pgdata`). Docker stores it in its own area on your disk; you never need to touch it directly.
`docker volume ls` lists your volumes. The data stays there through `down`, restarts and reboots,
and is deleted only by `down -v` or `docker volume rm`.

**"Do I need to install `sea-orm-cli` now?"**
No. It's a helper for SeaORM, and Part A3 installs it at the moment you first need it.

## Coming from another language?

Good news: this lesson is the same in every language. A Node, Python, Java or Go backend developer
uses the same Postgres, the same Docker and Compose, the same `psql` and the same curl (or Postman,
which is like Bruno). If you already have a Compose file for Postgres from another project, it
works the same way; only the port and names differ.

The `.env` file works like Node's `dotenv` package or Python's `python-dotenv`: a file of
`NAME=value` lines that's loaded into environment variables when your program starts, and never
committed to git. Rust has [crates](../glossary.md#crate) (libraries) that do the same job; you'll meet one later in
the book, when your code first reads `DATABASE_URL`.

## Common mistakes

**Docker isn't running.**

```text
failed to connect to the docker API at unix:///Users/you/.docker/run/docker.sock; check if the path is correct and if the daemon is running: dial unix /Users/you/.docker/run/docker.sock: connect: no such file or directory
```

The `docker` command is installed, but the Docker program in the background (the *daemon*) isn't
started. Older Docker versions word the same problem as `Cannot connect to the Docker daemon … Is
the docker daemon running?`. **Fix:** open Docker Desktop or OrbStack, wait until it says it's
running, and run your command again.

**Port 5433 is already taken.**

```text
Error response from daemon: failed to set up container networking: driver failed programming external connectivity on endpoint rust-book-backend-db-1 (252f1921b7030f176701a22ec8ca028e94d02106a92ada6f3fe9bf9d8bf804ae): Bind for 0.0.0.0:5433 failed: port is already allocated
```

Another program (often another project's database container) is already listening on port 5433.
**Fix:** either stop that other program, or give the book's database a different door. Change the
*left* number in `docker-compose.yml` (for example `"5440:5432"`), change the port in your `.env`
to match (`…@localhost:5440/rbh`), then run `docker compose up -d --wait` again. To find out who's
using the port, see "Check which program is using a port" in **More examples**.

**`psql` isn't installed.**

```text
zsh: command not found: psql
```

(In bash the same message reads `bash: psql: command not found`.) **Fix:** you don't need to
install it. Use the version inside the container:
`docker compose exec db psql -U postgres -d rbh -c 'select version();'`.

**The password is "wrong", even though it's `postgres`.**

```text
psql: error: connection to server at "localhost" (::1), port 5433 failed: FATAL:  password authentication failed for user "postgres"
```

Usually this means an **old volume** from an earlier attempt, created with a different password.
Remember, `POSTGRES_PASSWORD` is only read the first time, when the volume is empty; changing the
Compose file afterwards changes nothing. **Fix:** check the password in your URL first. If it's
right, start fresh (**this deletes all data in the book's database**):

```bash
docker compose down -v
docker compose up -d --wait
```

## More examples

### See what Postgres is saying

When something's wrong, the database's own log usually says why.

```bash
docker compose logs db
```

```text
…
db-1  | PostgreSQL init process complete; ready for start up.
db-1  | 
db-1  | 2026-09-24 16:28:02.077 UTC [1] LOG:  starting PostgreSQL 18.6 (Debian 18.6-1.pgdg13+2) on aarch64-unknown-linux-gnu, compiled by gcc (Debian 14.2.0-19) 14.2.0, 64-bit
db-1  | 2026-09-24 16:28:02.078 UTC [1] LOG:  listening on IPv4 address "0.0.0.0", port 5432
db-1  | 2026-09-24 16:28:02.078 UTC [1] LOG:  listening on IPv6 address "::", port 5432
db-1  | 2026-09-24 16:28:02.080 UTC [1] LOG:  listening on Unix socket "/var/run/postgresql/.s.PGSQL.5432"
db-1  | 2026-09-24 16:28:02.084 UTC [75] LOG:  database system was shut down at 2026-09-24 16:28:02 UTC
db-1  | 2026-09-24 16:28:02.086 UTC [1] LOG:  database system is ready to accept connections
```

`ready to accept connections` is the line you want. Inside the container, Postgres listens on
5432; the 5433 mapping lives outside it, in Docker.

### List the tables in your database

`\dt` is a `psql` shortcut (not SQL) that lists the tables; you'll use it a lot in Part A1.

```bash
psql postgres://postgres:postgres@localhost:5433/rbh -c '\dt'
```

```text
Did not find any tables.
```

That's correct for now: `rbh` is brand new and empty. Commands that start with a backslash (`\`)
are `psql`'s own shortcuts, and they don't need a `;`.

### Restart the database

If Postgres seems stuck, restarting it keeps all your data and only restarts the program.

```bash
docker compose restart db
```

```text
 Container rust-book-backend-db-1 Restarting 
 Container rust-book-backend-db-1 Started 
```

### Check which program is using a port

When you get `port is already allocated`, `lsof` (*list open files*, available on macOS and Linux)
tells you who's behind that door.

```bash
lsof -i :5433
```

```text
COMMAND     PID USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
OrbStack  72121  you   87u  IPv4 0x764f7c29cbd42c68      0t0  TCP *:5433 (LISTEN)
OrbStack  72121  you   88u  IPv6  0xc6b23331f0ab681      0t0  TCP *:5433 (LISTEN)
```

Here it's OrbStack (the Docker program) holding the port for a container. `-i :5433` means "network
connections on port 5433". To see *which* container, run `docker ps` and look for `:5433` in the
`PORTS` column. On Windows, `netstat -ano | findstr :5433` shows the same.

## Your turn

### 🟢 Guided

Start the database and ask Postgres to add one plus one. Fill in the blanks, then run both lines:

```bash
docker compose up -d ______
psql postgres://postgres:postgres@localhost:____/rbh -c 'select ______;'
```

<details><summary>Solution</summary>

```bash
docker compose up -d --wait
psql postgres://postgres:postgres@localhost:5433/rbh -c 'select 1 + 1;'
```

```text
 ?column? 
----------
        2
(1 row)
```

`--wait` makes sure Postgres is ready before `psql` connects, `5433` is the host port from the
Compose file, and `select 1 + 1;` asks Postgres to calculate a value. The column is called
`?column?` because we didn't give it a name. (No `psql`? Use
`docker compose exec db psql -U postgres -d rbh -c 'select 1 + 1;'`.)

</details>

### 🟡 Tweak

The `rbh` database was created for you. Change the command from the 🟢 exercise so that it creates a
second database called `playground`, then list all databases with the `psql` shortcut `\l` to see
it.

<details><summary>Solution</summary>

Change the SQL inside the quotes to `create database playground;`:

```bash
psql postgres://postgres:postgres@localhost:5433/rbh -c 'create database playground;'
psql postgres://postgres:postgres@localhost:5433/rbh -c '\l'
```

```text
CREATE DATABASE
                                                     List of databases
    Name    |  Owner   | Encoding | Locale Provider |  Collate   |   Ctype    | Locale | ICU Rules |   Access privileges   
------------+----------+----------+-----------------+------------+------------+--------+-----------+-----------------------
 playground | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | 
 postgres   | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | 
 rbh        | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | 
 template0  | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | =c/postgres          +
            |          |          |                 |            |            |        |           | postgres=CTc/postgres
 template1  | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | =c/postgres          +
            |          |          |                 |            |            |        |           | postgres=CTc/postgres
(5 rows)
```

`CREATE DATABASE` is Postgres confirming it worked. One Postgres server can hold many databases:
`playground` and `rbh` are yours, `postgres` is a default one, and `template0`/`template1` are
templates Postgres copies when it creates a new database. (Command-line fans may know the
`createdb` program, which does the same thing; the SQL form works everywhere.)

</details>

### 🔴 From scratch

Prove that your data survives. Stop everything, start it again, and show that the `playground`
database from the 🟡 exercise is still there. Write the commands yourself.

<details><summary>Solution</summary>

```bash
docker compose down
docker compose up -d --wait
psql postgres://postgres:postgres@localhost:5433/rbh -c '\l playground'
```

The `up` output this time has no `Volume … Created` lines, because the volume already exists:

```text
 Network rust-book-backend_default Creating 
 Network rust-book-backend_default Created 
 Container rust-book-backend-db-1 Creating 
 Container rust-book-backend-db-1 Created 
 Container rust-book-backend-db-1 Starting 
 Container rust-book-backend-db-1 Started 
 Container rust-book-backend-db-1 Waiting 
 Container rust-book-backend-db-1 Healthy 
```

and `\l playground` (list only databases with that name) finds it:

```text
                                                   List of databases
    Name    |  Owner   | Encoding | Locale Provider |  Collate   |   Ctype    | Locale | ICU Rules | Access privileges 
------------+----------+----------+-----------------+------------+------------+--------+-----------+-------------------
 playground | postgres | UTF8     | libc            | en_US.utf8 | en_US.utf8 |        |           | 
(1 row)
```

**Why it survived:** `down` threw away the *container*, but the database's files live in the
`pgdata` **volume**, which `down` keeps. The new container attached the same volume at
`/var/lib/postgresql` and found everything where it was left. If you had used `down -v`, the
volume would have been deleted and `\l playground` would find nothing. You can check that the
volume is still there with `docker volume ls`.

</details>

## Quick check

<div class="quiz" data-topic="your-toolbox"></div>

## Remember this

- Docker runs Postgres 18 in a container, so nothing is installed into your system; the book's
  `docker-compose.yml` describes it.
- `"5433:5432"` means *your computer's* port 5433 leads to port 5432 *inside* the container. We use
  5433 so it never clashes with a Postgres you already have.
- A Postgres URL has the shape `postgres://USER:PASSWORD@HOST:PORT/DATABASE`; the book's lives in
  `DATABASE_URL` in `.env`, which is never committed.
- Start with `docker compose up -d --wait`, stop with `docker compose down`. **`down -v` deletes
  your data.**
- No `psql`? `docker compose exec db psql -U postgres -d rbh` works everywhere.

## Go deeper

- [Rust for Humans: Install Rust](https://open-source-bd.github.io/rustbook-for-human/start-here/install-rust.html)
- [Rust for Humans: Cargo basics](https://open-source-bd.github.io/rustbook-for-human/start-here/cargo-basics.html)
- [Docker Desktop](https://docs.docker.com/get-started/get-docker/) — Install Docker.
- [Postgres Docker image](https://hub.docker.com/_/postgres) — Options for the postgres image.
- [OrbStack](https://orbstack.dev/) — A lighter Docker alternative for macOS.
- [Docker Compose file reference](https://docs.docker.com/reference/compose-file/) — Every key a
  Compose file can contain.
- [psql documentation](https://www.postgresql.org/docs/18/app-psql.html) — Every `psql` option and
  backslash shortcut.
- [Bruno](https://www.usebruno.com/) — The optional desktop app for sending requests.

**Next:**

- [How a web backend works](../part-0-start/how-a-web-backend-works.md)
