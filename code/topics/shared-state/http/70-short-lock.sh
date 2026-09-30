# serve: -p shared-state --example state-short-lock
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Dune"}'
curl -i http://127.0.0.1:3000/books
