# serve: -p custom-extractors --example extract-new-key
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/secret
curl -i -H 'x-api-key: open-sesame' http://127.0.0.1:3000/secret
