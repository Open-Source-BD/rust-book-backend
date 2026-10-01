# serve: -p testing-handlers --example th-delete-notes
curl -i -X POST http://127.0.0.1:3000/notes -H 'content-type: application/json' -d '{"text":"one"}'
curl -i -X DELETE http://127.0.0.1:3000/notes
curl -i http://127.0.0.1:3000/notes
