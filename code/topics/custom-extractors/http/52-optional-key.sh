# serve: -p custom-extractors --example extract-optional-key
curl -i http://127.0.0.1:3000/greeting
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/greeting
curl -i -H 'x-api-key: guess' http://127.0.0.1:3000/greeting
