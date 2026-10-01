# serve: -p input-validation --example validate-must-match
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada","password":"correct horse","password_again":"correct horse"}'
curl -i -X POST http://127.0.0.1:3000/sign-up -H 'content-type: application/json' -d '{"username":"ada","password":"correct horse","password_again":"correct hose"}'
