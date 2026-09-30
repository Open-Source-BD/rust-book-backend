# serve: -p path-and-query-extractors --example extractors-query-map
curl -i "http://127.0.0.1:3000/filters?color=red&size=big"
curl -i http://127.0.0.1:3000/filters
curl -i "http://127.0.0.1:3000/filters?anything=goes"
