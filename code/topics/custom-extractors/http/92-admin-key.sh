# serve: -p custom-extractors --example extract-admin-key
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/admin
curl -i -H 'x-api-key: letmein' -H 'x-role: admin' http://127.0.0.1:3000/admin
curl -i -H 'x-role: admin' http://127.0.0.1:3000/admin
curl -i -H 'x-api-key: letmein' http://127.0.0.1:3000/secret
