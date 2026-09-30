# serve: -p json-and-serde --example json-deny-unknown
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Emma","author":"Jane Austen"}'
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Emma"}'
