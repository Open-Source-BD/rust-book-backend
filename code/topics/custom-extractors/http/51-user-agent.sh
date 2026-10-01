# serve: -p custom-extractors --example extract-user-agent
curl -i -H 'user-agent: book-reader/1.0' http://127.0.0.1:3000/whoami
curl -i -H 'user-agent:' http://127.0.0.1:3000/whoami
