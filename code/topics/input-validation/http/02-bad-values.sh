# serve: -p input-validation
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"al","email":"nope","age":9}'
