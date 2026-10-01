# serve: -p input-validation --example validate-option-mistake
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada"}'
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada","email":"nope"}'
