# How to use this book

> **Beginner** · Part 0 · Before you start

## By the end of this lesson

- You know what you will be able to build after this book.
- You can check that you know enough Rust to start.
- You know how every lesson is laid out and how to use it.

## What & why

What this book teaches, what Rust you need first, and how each lesson page works.

Think about a restaurant. The dining room is the part you see: the tables, the menu, the waiter.
The kitchen is the part you never see: it keeps the food in the fridge, follows the recipes, checks
that nobody gets a dish they are allergic to, and sends plates back out. Every order goes through
the kitchen, even though no customer ever walks into it.

Apps work the same way. The screen you tap or click is the dining room. The
[**backend**](../glossary.md#backend) is the kitchen: the part of an app that runs on a
[**server**](../glossary.md#server) (a program on some computer that waits for questions and sends
back answers) instead of on your own phone or laptop. It stores the data, applies the rules ("you
can't buy more than we have in stock"), and sends back the answer. This book teaches you to build
that kitchen.

**Why build it in Rust?** Three reasons that matter for a backend:

- **It's fast.** A Rust backend answers quickly and handles many people at once.
- **It's cheap to run.** Rust programs use little memory, so a small, cheap server can carry a lot
  of traffic.
- **The compiler catches bugs before your users do.** Many mistakes that would crash a Node or
  Python backend at 3 a.m. (a missing value, a wrong type, two tasks changing the same data) are
  refused by the Rust compiler before the program ever runs.

This first lesson makes sure you're ready: it checks the Rust you need, checks that Rust is
installed, and shows you how every lesson page is built so you always know where to look.

## The idea, slowly

### Step 1: the Rust you need

This book does not re-teach the Rust language. It is the sequel to
[Rust for Humans](https://open-source-bd.github.io/rustbook-for-human/), and it assumes you are
comfortable with the six ideas below. Read the list and, for each one, ask yourself: *"Could I
explain this to a friend in two sentences?"* If the answer is no, open the linked lesson first. It
takes an afternoon, and it will save you days of confusion later.

- [ ] **Ownership and borrowing**: every value has one owner; you lend it with `&` or `&mut`.
  → [Ownership](https://open-source-bd.github.io/rustbook-for-human/ownership/ownership.html),
  [Borrowing](https://open-source-bd.github.io/rustbook-for-human/ownership/borrowing.html)
- [ ] **`struct` and `enum`**: bundling data together, and "one of several choices".
  → [Structs](https://open-source-bd.github.io/rustbook-for-human/language-basics/structs.html),
  [Enums](https://open-source-bd.github.io/rustbook-for-human/language-basics/enums.html)
- [ ] **`Result` and `?`**: a function that can fail returns `Ok(value)` or `Err(error)`, and `?`
  passes the error up to the caller.
  → [Result and Option](https://open-source-bd.github.io/rustbook-for-human/abstractions/result-and-option.html),
  [The question mark operator](https://open-source-bd.github.io/rustbook-for-human/abstractions/the-question-mark-operator.html)
- [ ] **Traits**: a named set of abilities a type promises to have (like `Display` or `Clone`).
  → [Traits basics](https://open-source-bd.github.io/rustbook-for-human/abstractions/traits-basics.html)
- [ ] **Closures**: small unnamed functions you can store or pass around, written `|x| x + 1`.
  → [Closures](https://open-source-bd.github.io/rustbook-for-human/abstractions/closures.html)
- [ ] **`async fn` and `.await`**: [async](../glossary.md#async) functions can pause while they wait
  for something slow.
  → [Async basics](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/async-basics.html)

You don't need to be an expert in any of these. "I've seen it, I roughly get it, I know where to
look it up" is enough. Every time one of them shows up in this book, the lesson gives you a
one-sentence reminder and a link back.

### Step 2: check that Rust is installed

Before any backend code, let's make sure your computer has the two Rust tools the whole book
relies on. Open a terminal (on macOS: the *Terminal* app; on Windows: *PowerShell*; on Linux: your
usual terminal) and type these two commands:

```bash
rustc --version
cargo --version
```

### Line by line

`rustc --version`
- **What:** asks the Rust compiler to print its version number and stop.
- **Why:** `rustc` is the **compiler**, the program that turns your Rust source code into a program
  your computer can run. If it's missing or too old, nothing else in the book will build.
- **How:** `rustc` is the program's name; `--version` is an **option** (an extra instruction you
  add after a command's name) that means "don't compile anything, only tell me which version you
  are".
- **Remove it and…** you won't know whether Rust is installed until a later lesson fails with a
  confusing error.

`cargo --version`
- **What:** asks Cargo to print its version number.
- **Why:** `cargo` is Rust's **build tool and package manager**. It creates projects, downloads the
  libraries your project uses (Rust calls a library a [**crate**](../glossary.md#crate)), calls
  `rustc` for you, and runs your program. In practice you type `cargo` all day and almost never
  type `rustc` yourself.
- **How:** same idea as above: the tool's name, then `--version`.
- **Remove it and…** you won't find out that Cargo is missing (or older than `rustc`) until
  `cargo run` fails in the next lesson.

### Run it

```bash
rustc --version
cargo --version
```

```text
rustc 1.96.0 (ac68faa20 2026-05-25)
cargo 1.96.0 (30a34c682 2026-05-25)
```

How to read `rustc 1.96.0 (ac68faa20 2026-05-25)`: `1.96.0` is the version (major 1, minor 96,
patch 0). Inside the brackets, `ac68faa20` is a short code that identifies the exact source the
compiler was built from, and `2026-05-25` is the date it was released. Your numbers will probably
be different, and that's fine.

- ✅ If you see two version lines, you're right: **any version 1.85 or newer works (edition 2024)**.
  An **edition** is a set of language rules Rust can opt into; the projects in this book use
  edition 2024, which first shipped in Rust 1.85.
- ❌ If you see `command not found` (or, on Windows, `is not recognized as the name of a cmdlet`),
  Rust isn't installed or your terminal can't find it. Install it by following
  [Rust for Humans: Install Rust](https://open-source-bd.github.io/rustbook-for-human/start-here/install-rust.html),
  then **close the terminal and open a new one** so it picks up the new tools.
- ❌ If your version is older than 1.85, run `rustup update` and check again.

### Step 3: the anatomy of a lesson page

Every lesson in this book (including this one) has the same eleven sections in the same order.
Once you know them, you always know where to look:

1. **By the end of this lesson**: two or three things you'll be able to *do*. Use them as a
   checklist when you finish.
2. **What & why**: the real problem the lesson solves, with an everyday analogy, before any code.
3. **The idea, slowly**: the lesson itself, in small steps. After each piece of code come two
   helper sections:
   - **Line by line**: every line that matters, broken down into *What* it does, *Why* it's
     there, *How* it works, and *Remove it and…* (what breaks without it).
   - **Run it**: the exact command to type, the exact output you should see, and a ✅ / ❌ pair
     that tells you whether it worked and what to do if it didn't.
4. **You might be wondering…**: answers to the questions most people ask at this point.
5. **Coming from another language?**: how the idea maps to JavaScript, Python, Java or Go, if
   you know one of them. If you don't, skip it.
6. **Common mistakes**: real error messages, what they mean, and how to fix them.
7. **More examples**: four short, real-world uses of the same idea.
8. **Your turn**: three exercises, getting harder: 🟢 *Guided*, 🟡 *Tweak* and 🔴 *From scratch*.
   Each has a hidden solution.
9. **Quick check**: a short quiz on this lesson only.
10. **Remember this**: the three to five things worth keeping.
11. **Go deeper**: official documentation and Rust for Humans links, if you want more.

## You might be wondering…

**"Do I need to know SQL?"**
No. [**SQL**](../glossary.md#sql) is the language you use to talk to a
[**database**](../glossary.md#database) (an organised store of data that keeps it safe even when
programs stop). Part A1 teaches it from zero before any Rust code touches a database.

**"Do I need to build a frontend (the screens people see)?"**
No. This book builds the kitchen only. To test your backend you'll use `curl`, a small command-line
program that sends a [**request**](../glossary.md#request) to a server and prints the
[**response**](../glossary.md#response). The next lesson installs it.

**"Why not build this in Node or Python instead?"**
You can, and many people do. Node and Python are quicker to start with; Rust asks you to be more
careful up front. The payoff is a backend that uses less memory, answers faster, and refuses to
compile many bugs that Node or Python would only reveal while real users are clicking. If you've
already learned the Rust basics, a backend is one of the most rewarding places to use them.

**"What if I get stuck?"**
Three places to look, in this order: the **Common mistakes** section of the lesson (most errors
people hit are listed there with their fix), the **Line by line** section (re-read the line that
confuses you), and the full, working code for every listing, which lives in the `code/` folder of
the book's repository. You can compare your code with it line by line.

**"Do I have to read it in order?"**
Yes, the first time. Each lesson uses only what earlier lessons taught, so skipping ahead is the
fastest way to meet words nobody has explained to you yet.

## Coming from another language?

If you've built a backend before, here is how this book's tools line up with what you know. The
**web [framework](../glossary.md#framework)** (a library that does the repetitive parts of a web
server for you, so you only write what's special to your app) handles incoming requests; the
[**ORM**](../glossary.md#orm) (a library that lets you use database rows as normal objects instead
of writing SQL by hand) handles the database.

| You know… | Web framework | Database library (ORM) | In this book |
|---|---|---|---|
| Node.js | Express | Prisma | Axum + SeaORM |
| Python | FastAPI | SQLAlchemy | Axum + SeaORM |
| Java | Spring Boot | JPA (Hibernate) | Axum + SeaORM |
| Go | net/http | GORM | Axum + SeaORM |

The ideas carry over directly: [routes](../glossary.md#route) (rules that say which code answers
which address), [handlers](../glossary.md#handler) (the functions that answer them) and database
models (structs that mirror your tables). What changes is that Rust checks far more of it at compile time, before the server ever
starts.

## Common mistakes

- **Skipping Part A and jumping to the capstone.** The capstone (Part B, the big final project where you build a whole online shop's backend) moves fast because it
  assumes every Part A lesson. If you start there, you'll meet dozens of unexplained ideas at once.
  Fix: go in order. If you already know SQL, you can skim Part A1, but still do its **Your turn**
  exercises to prove it.
- **Copying code without running it.** Reading code feels like understanding it, but it isn't the
  same. Fix: type (or paste) every listing, run it, and compare your output with the **Run it**
  section. When they differ, you've found something worth learning.
- **Reading without doing the Your turn exercises.** The exercises are where the idea moves from
  "I followed along" to "I can do it myself". Fix: always do at least the 🟢 and 🟡 tiers before
  moving on, and try the 🔴 one before opening its solution.

## More examples

### How to use a Line by line block

When one line of a listing confuses you, find that line in **Line by line** and read its four
parts in order. Here is what one entry looks like:

```markdown
`cargo --version`
- **What:** asks Cargo to print its version number.
- **Why:** Cargo builds and runs every project in this book.
- **How:** the tool's name, then the `--version` option.
- **Remove it and…** you won't know Cargo is missing until later.
```

Read the **Remove it and…** part twice: knowing what breaks without a line is the fastest way to
understand why it's there.

### How to read a Run it block

A **Run it** block always has the same three parts: the command, the output, and how to judge it.

```bash
cargo --version
```

```text
cargo 1.96.0 (30a34c682 2026-05-25)
```

Type the command yourself, then compare *the shape* of your output, not every character: version
numbers, dates, times and file paths will differ on your computer. What matters is that you got a
version line and not an error.

### How to use the full-code link

Code that belongs to a bigger program (every Axum and SeaORM listing, from Part A2 on) comes with
a line like this under it:

```text
📁 Full code: code/topics/tour-of-the-stack
▶ Run it: cargo run -p tour-of-the-stack
```

The path points into the `code/` folder of the book's repository, and the run command is typed from
inside that `code/` folder. Open it when you want to see the
listing in its full file, with every `use` line and the `Cargo.toml` around it. The book's
automatic checks compile and test that folder on every change, so the code there is always known to
work.

### How to use the Review page

The [Review & flashcards](../review.md) page collects the key cards from every lesson into one
deck. Each card looks like this:

```text
Front:  What are the three exercise tiers?
        [ Flip ]
Back:   🟢 Guided (fill in), 🟡 Tweak (change working code),
        🔴 From scratch (write from a one-line spec).
        [ Got it ]   [ Shaky ]
```

Answer in your head *before* you press **Flip**. Press **Shaky** when you weren't sure: those cards
come back first next time. Five minutes there before each new lesson keeps old ideas from fading.

## Your turn

### 🟢 Guided

Open a terminal and run the two version commands from Step 2. Fill in the blanks with what your
computer printed:

```text
rustc ____ (_________ __________)
cargo ____ (_________ __________)
```

Is your version 1.85 or newer?

<details><summary>Solution</summary>

On the computer used to write this book, the output was:

```text
rustc 1.96.0 (ac68faa20 2026-05-25)
cargo 1.96.0 (30a34c682 2026-05-25)
```

So the blanks are: the version (`1.96.0`), the short build code (`ac68faa20`), and the release
date (`2026-05-25`). Your numbers can be different. If the version is 1.85 or newer, you're ready.
If it's older, run `rustup update`. If you got `command not found`, install Rust first (see the ❌
line in Step 2's **Run it**).

</details>

### 🟡 Tweak

Now let Cargo do real work. In a folder where you keep your projects, create a new project called
`hello-backend` and run it:

```bash
cargo new hello-backend
cd hello-backend
cargo run
```

Then change the message it prints to `Hello, backend!` and run it again.

<details><summary>Solution</summary>

`cargo new hello-backend` creates a folder with a `Cargo.toml` (the project's settings file) and a
`src/main.rs` (the code). `cd hello-backend` moves your terminal into that folder, and `cargo run`
compiles and runs it:

```text
    Creating binary (application) `hello-backend` package
note: see more `Cargo.toml` keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html
   Compiling hello-backend v0.1.0 (/path/to/hello-backend)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.86s
     Running `target/debug/hello-backend`
Hello, world!
```

(The path in brackets is wherever you created the folder, and the time will differ.) Open
`Cargo.toml` and you'll see `edition = "2024"`, the edition this book uses. To change the message,
edit `src/main.rs`:

```rust,editable
fn main() {
    println!("Hello, backend!");
}
```

Run `cargo run` again and the last line becomes `Hello, backend!`.

</details>

### 🔴 From scratch

No code this time. Write down, in one sentence each, **three backends you'd like to be able to
build** after this book. Be concrete: say what data it stores and who uses it. Keep the list; at
the end of Part A, check which ones you could now build.

<details><summary>Solution</summary>

There's no single right answer. Here are some sample answers to show the level of detail that
helps:

- "A booking API for my friend's barber shop that stores appointments and stops two customers from
  booking the same time slot."
- "A backend for a family recipe app that stores recipes and lets each person mark their
  favourites."
- "An API for my football club that stores players, matches and scores, and returns the league
  table."

Notice each one names the **data** (appointments, recipes, players) and a **rule** (no double
bookings, per-person favourites, a computed table). Data plus rules is exactly what a backend is
for, and by the end of this book you'll have built all of these pieces.

</details>

## Quick check

<div class="quiz" data-topic="how-to-use-this-book"></div>

## Remember this

- A backend is the kitchen of an app: it runs on a server, stores the data, applies the rules and
  sends back answers.
- You need basic Rust first: ownership and borrowing, `struct`/`enum`, `Result` and `?`, traits,
  closures, and `async`/`.await`. Each links back to Rust for Humans.
- `rustc --version` and `cargo --version` check your setup; any version 1.85 or newer works.
- Every lesson has the same eleven sections; **Line by line** explains each line and **Run it**
  shows the exact output to expect.
- Run every listing and do every **Your turn**. Doing it is how it sticks.

## Go deeper

- [Rust for Humans: Ownership](https://open-source-bd.github.io/rustbook-for-human/ownership/ownership.html)
- [Rust for Humans: Result and Option](https://open-source-bd.github.io/rustbook-for-human/abstractions/result-and-option.html)
- [Rust for Humans: Async basics](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/async-basics.html)
- [Rust for Humans: Install Rust](https://open-source-bd.github.io/rustbook-for-human/start-here/install-rust.html)
- [The Rust Book](https://doc.rust-lang.org/book/) — The official Rust guide.
- [Rust editions](https://doc.rust-lang.org/edition-guide/) — What an edition is and what 2024 changed.

**Next:**

- [Your toolbox](../part-0-start/your-toolbox.md)
