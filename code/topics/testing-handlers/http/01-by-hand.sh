# serve: -p testing-handlers
curl -i http://127.0.0.1:3000/health
curl -i -X POST http://127.0.0.1:3000/notes -H 'content-type: application/json' -d '{"text":"buy milk"}'
curl -i -X POST http://127.0.0.1:3000/notes -H 'content-type: application/json' -d '{"text":"two"}'
curl -i http://127.0.0.1:3000/notes
curl -i -X POST http://127.0.0.1:3000/notes -H 'content-type: application/json' -d '{"words":"oops"}'
