// Single source of truth for every page in the book.
// status "draft"  -> shown greyed-out in the sidebar, no stub written, not validated.
// status "published" -> stub generated (if missing), linked in sidebar, fully validated.
const RFH = "https://open-source-bd.github.io/rustbook-for-human/";
const rfh = (label, path) => ({ label: `Rust for Humans: ${label}`, href: RFH + path });

const draft = (slug, title, part, kind = "lesson", level = "Intermediate") => ({
  slug, title, part, kind, level, status: "draft",
});

export default [
  // ---- Part 0 · Before you start ----
  {
    slug: "how-to-use-this-book",
    title: "How to use this book",
    part: "0", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "You know what you will be able to build after this book.",
      "You can check that you know enough Rust to start.",
      "You know how every lesson is laid out and how to use it.",
    ],
    summary: "What this book teaches, what Rust you need first, and how each lesson page works.",
    prereq: [],
    next: ["your-toolbox"],
    rfhLinks: [rfh("Ownership", "ownership/ownership.html"), rfh("Result and Option", "abstractions/result-and-option.html"), rfh("Async basics", "runtime-and-ecosystem/async-basics.html")],
    links: [{ label: "The Rust Book", href: "https://doc.rust-lang.org/book/", note: "The official Rust guide." }],
  },
  {
    slug: "your-toolbox",
    title: "Your toolbox",
    part: "0", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "Rust, Docker and an HTTP client are installed and checked.",
      "A Postgres database is running in Docker on your computer.",
      "You can connect to it and run your first SQL query.",
    ],
    summary: "Install and check every tool the book uses, and start your own Postgres database.",
    prereq: ["how-to-use-this-book"],
    next: ["how-a-web-backend-works"],
    rfhLinks: [rfh("Install Rust", "start-here/install-rust.html"), rfh("Cargo basics", "start-here/cargo-basics.html")],
    links: [
      { label: "Docker Desktop", href: "https://docs.docker.com/get-started/get-docker/", note: "Install Docker." },
      { label: "Postgres Docker image", href: "https://hub.docker.com/_/postgres", note: "Options for the postgres image." },
    ],
  },
  {
    slug: "how-a-web-backend-works",
    title: "How a web backend works",
    part: "0", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "You can explain what happens between typing a URL and seeing a result.",
      "You can read an HTTP request and response line by line.",
      "You know what JSON, REST and status codes are, and can send a request with curl.",
    ],
    summary: "Clients, servers, HTTP requests and responses, JSON, REST and status codes — the ideas every later lesson uses.",
    prereq: ["your-toolbox"],
    next: ["tour-of-the-stack"],
    rfhLinks: [rfh("Serde and JSON", "runtime-and-ecosystem/serde-and-json.html")],
    links: [
      { label: "MDN: An overview of HTTP", href: "https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Overview", note: "A friendly deep dive." },
      { label: "MDN: HTTP status codes", href: "https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status", note: "Every code explained." },
    ],
  },
  {
    slug: "tour-of-the-stack",
    title: "Tour of the stack",
    part: "0", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "You know what Tokio, Axum, SeaORM and Postgres each do.",
      "You can follow one request through all four layers.",
      "You have run your first Axum server and seen it answer.",
    ],
    summary: "Meet the four tools of this book, see how one request flows through them, and run your first Axum server: under twenty lines of Rust.",
    prereq: ["how-a-web-backend-works"],
    next: ["what-is-a-database"],
    codeDir: "code/topics/tour-of-the-stack",
    rfhLinks: [rfh("Async basics", "runtime-and-ecosystem/async-basics.html"), rfh("Tokio runtime and tasks", "runtime-and-ecosystem/tokio-runtime-and-tasks.html"), rfh("Web services", "runtime-and-ecosystem/web-services.html")],
    links: [
      { label: "Axum docs", href: "https://docs.rs/axum/0.8.9/axum/", note: "Official API reference." },
      { label: "Tokio tutorial", href: "https://tokio.rs/tokio/tutorial", note: "How the async runtime works." },
      { label: "SeaORM docs", href: "https://www.sea-ql.org/SeaORM/docs/index/", note: "Official guide." },
    ],
  },

  // ---- Part A1 · PostgreSQL & SQL ----
  {
    slug: "what-is-a-database",
    title: "What is a database?",
    part: "A1", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "You can say what a database is and why apps don't keep their data in plain files.",
      "You know what a table, a row and a column are.",
      "You have run your first two SQL queries against your own Postgres.",
    ],
    summary: "Why apps store data in a database, what Postgres and SQL are, and your first queries.",
    prereq: ["tour-of-the-stack"],
    next: ["tables-rows-and-psql"],
    codeDir: "code/sql/what-is-a-database",
    rfhLinks: [],
    links: [
      { label: "PostgreSQL tutorial", href: "https://www.postgresql.org/docs/18/tutorial.html", note: "The official beginner's tour of Postgres." },
      { label: "SQL Language", href: "https://www.postgresql.org/docs/18/sql.html", note: "The full reference for Postgres's SQL." },
    ],
  },
  {
    slug: "tables-rows-and-psql",
    title: "Tables, rows and psql",
    part: "A1", kind: "lesson", level: "Beginner", status: "published",
    outcomes: [
      "You can create a table with named, typed columns.",
      "You can insert rows and select exactly the columns you want.",
      "You can find your way around with psql's backslash commands.",
    ],
    summary: "Create tables, add and read rows, and explore your database with psql's backslash commands.",
    prereq: ["what-is-a-database"],
    next: ["postgres-data-types"],
    codeDir: "code/sql/tables-rows-and-psql",
    rfhLinks: [],
    links: [
      { label: "psql", href: "https://www.postgresql.org/docs/18/app-psql.html", note: "Every psql option and backslash command." },
      { label: "CREATE TABLE", href: "https://www.postgresql.org/docs/18/sql-createtable.html", note: "The full reference for creating tables." },
    ],
  },
  draft("postgres-data-types", "Postgres data types", "A1", "lesson", "Beginner"),
  draft("crud-in-sql", "CRUD in SQL", "A1", "lesson", "Beginner"),
  draft("keys-and-relations", "Keys and relations", "A1", "lesson", "Beginner"),
  draft("indexes", "Indexes", "A1", "lesson", "Beginner"),
  draft("sql-transactions", "SQL transactions", "A1", "lesson", "Beginner"),
  draft("a1-build-library-schema", "Build it: a library database", "A1", "project", "Beginner"),
  draft("cheatsheet-sql", "Cheat sheet: SQL", "A1", "cheatsheet", "Beginner"),

  // ---- Part A2 · Axum ----
  draft("hello-axum", "Hello, Axum", "A2", "lesson", "Beginner"),
  draft("routes-and-methods", "Routes and HTTP methods", "A2", "lesson", "Beginner"),
  draft("handlers-and-into-response", "Handlers and IntoResponse", "A2", "lesson", "Beginner"),
  draft("path-and-query-extractors", "Path and Query extractors", "A2", "lesson", "Beginner"),
  draft("json-and-serde", "JSON with serde", "A2", "lesson", "Beginner"),
  draft("shared-state", "Shared state", "A2"),
  draft("error-handling-in-axum", "Error handling in Axum", "A2"),
  draft("middleware-and-tower-layers", "Middleware and Tower layers", "A2"),
  draft("nesting-and-modular-routers", "Nesting and modular routers", "A2"),
  draft("custom-extractors", "Custom extractors", "A2"),
  draft("input-validation", "Input validation", "A2"),
  draft("testing-handlers", "Testing handlers", "A2"),
  draft("a2-build-todo-api", "Build it: a Todo API", "A2", "project", "Beginner"),
  draft("cheatsheet-axum", "Cheat sheet: Axum", "A2", "cheatsheet"),

  // ---- Part A3 · SeaORM ----
  draft("what-is-an-orm", "What is an ORM?", "A3", "lesson", "Beginner"),
  draft("connecting-to-postgres", "Connecting to Postgres", "A3", "lesson", "Beginner"),
  draft("migrations", "Migrations", "A3"),
  draft("generating-entities", "Generating entities", "A3"),
  draft("entity-model-activemodel-column", "Entity, Model, ActiveModel and Column", "A3"),
  draft("inserting-rows", "Inserting rows", "A3"),
  draft("selecting-rows", "Selecting rows", "A3"),
  draft("update-and-delete", "Update and delete", "A3"),
  draft("relations-and-loading", "Relations and loading", "A3"),
  draft("seaorm-transactions", "Transactions in SeaORM", "A3"),
  draft("raw-sql-and-custom-selects", "Raw SQL and custom selects", "A3"),
  draft("testing-with-seaorm", "Testing with SeaORM", "A3"),
  draft("a3-build-blog-data", "Build it: a blog data layer", "A3", "project"),
  draft("cheatsheet-seaorm", "Cheat sheet: SeaORM", "A3", "cheatsheet"),

  // ---- Part A4 · Putting it together ----
  draft("config-and-env", "Config and .env", "A4"),
  draft("logging-with-tracing", "Logging with tracing", "A4"),
  draft("project-layout", "Project layout", "A4"),
  draft("mini-notes-api", "Build it: a Notes API", "A4", "project"),
  draft("a4-build-url-shortener", "Build it yourself: a URL shortener", "A4", "project"),
  draft("ready-for-the-capstone", "Ready for the capstone?", "A4", "checklist"),
  draft("cheatsheet-project-patterns", "Cheat sheet: project patterns", "A4", "cheatsheet"),

  // ---- Part B · Capstone: ShopRS ----
  // kind for capstone chapters is decided in the Phase 4 plan; drafts are not validated.
  draft("shop-01-plan", "ShopRS 1: Plan the shop", "B"),
  draft("shop-02-skeleton", "ShopRS 2: Project skeleton", "B"),
  draft("shop-03-schema", "ShopRS 3: Database schema", "B"),
  draft("shop-04-errors-and-responses", "ShopRS 4: Errors and responses", "B"),
  draft("shop-05-register", "ShopRS 5: Register users", "B"),
  draft("shop-06-login-jwt", "ShopRS 6: Log in with JWT", "B"),
  draft("shop-07-roles", "ShopRS 7: Admin roles", "B"),
  draft("shop-08-categories", "ShopRS 8: Categories", "B"),
  draft("shop-09-products", "ShopRS 9: Products", "B"),
  draft("shop-10-product-listing", "ShopRS 10: Listing, filters and search", "B"),
  draft("shop-11-cart", "ShopRS 11: Shopping cart", "B"),
  draft("shop-12-checkout", "ShopRS 12: Checkout", "B"),
  draft("shop-13-payments", "ShopRS 13: Payments", "B"),
  draft("shop-14-orders", "ShopRS 14: Orders", "B"),
  draft("shop-15-hardening", "ShopRS 15: Production hardening", "B"),
  draft("shop-16-integration-tests", "ShopRS 16: Integration tests", "B"),
  draft("shop-17-openapi", "ShopRS 17: API docs with OpenAPI", "B"),
  draft("shop-18-ship-it", "ShopRS 18: Ship it", "B"),
];
