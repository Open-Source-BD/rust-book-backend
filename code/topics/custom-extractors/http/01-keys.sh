# serve: -p custom-extractors
curl -i http://127.0.0.1:3000/public
curl -i http://127.0.0.1:3000/secret
curl -i -H 'x-api-key: guess' http://127.0.0.1:3000/secret
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/secret
