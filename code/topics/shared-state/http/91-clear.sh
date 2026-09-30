# serve: -p shared-state --example state-clear
curl -i -X POST http://127.0.0.1:3000/books -H 'content-type: application/json' -d '{"title":"Dune"}'
curl -i -X DELETE http://127.0.0.1:3000/books
curl -i http://127.0.0.1:3000/books
