# Unofficial Bitrix Websocker server

Неофициальный websocket сервер для локальной разработки на Битрикс24в docker-контейнерах.

## Как использовать docker образ?

>Инструкция подготовлена для среды [aclips/bitrix24-docker](https://github.com/aclips/bitrix24-docker), но сервер может быть использован для любого образа.

Для этого необходимо сделать 3 простых шага:

1. Сконфигурировать nginx-прокси для websocket запросов.

Для этого в http-сецию конфигурацинного файла nginx добавить:
```
upstream push-upstream {
    server push:9099;
}

map $http_upgrade $connection_upgrade {
  default upgrade;
  '' 'close';
}

map $http_upgrade  $replace_upgrade {
  default $http_upgrade;
  ''      "websocket";
}
```

В server-секцию добавить:
```
location ~* ^/bitrix/subws/ {
    proxy_pass http://push-upstream;
    # http://blog.martinfjordvald.com/2013/02/websockets-in-nginx/
    # 12h+0.5
    proxy_max_temp_file_size 0;
    proxy_read_timeout  43800;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $replace_upgrade;
    proxy_set_header Connection $connection_upgrade;
}

location ~* ^/bitrix/sub/ {
    rewrite ^/bitrix/sub/(.*)$ /bitrix/subws/$1 break;
    proxy_pass http://push-upstream;
    proxy_max_temp_file_size 0;
    proxy_read_timeout  43800;
}

location ~* ^/bitrix/rest/ {
    proxy_pass http://push-upstream;
    proxy_max_temp_file_size 0;
    proxy_read_timeout  43800;
}
```


2. В `docker-compose.yml` файл добавить:

```
  push:
    build: 
      context: ./containers/push
      dockerfile: ./Dockerfile
    container_name: ${PROJECT_PREFIX}_push
```

3. Разместить в фапке `./containers/push` файлы:

`Dockerfile` с содержимым:

```
FROM gromdron/bitrix-push-workspace:v1.0

WORKDIR /opt/push-server

EXPOSE 9099

COPY push_config.toml .

ENV CONFIG_FILE=/opt/push-server/push_config.toml

CMD ["push-server"]
```

и файл push-server.toml из корня репозитория.

>Файл `push-server.toml` можно конифгурировать исходя из требований проекта