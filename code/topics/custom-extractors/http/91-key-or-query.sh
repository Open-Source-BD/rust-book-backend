# serve: -p custom-extractors --example extract-key-or-query
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/secret
curl -i 'http://127.0.0.1:3000/secret?key=letmein'
curl -i 'http://127.0.0.1:3000/secret?key=guess'
curl -i http://127.0.0.1:3000/secret
