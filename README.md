# Unofficial Bitrix WebSocket server

Push-сервер это компонент Битрикс24 для поддержки обновлений в реальном времени (чат, уведомления, CRM). Текущий проект это замена — один бинарник без внешних зависимостей.

## Быстрый старт

Минимальный запуск за 2 минуты:

```bash
docker run -d --name push -p 9099:9099 gromdron/bitrix-push-workspace:v2.0
```

Или через `docker-compose.yml`:

```yaml
services:
  push:
    image: gromdron/bitrix-push-workspace:v2.0
    container_name: push
    ports:
      - "9099:9099"
```

После запуска в настройках Битрикс24 (*Настройки → Настройки модулей → Push and Pull*) укажите "Путь для публикации команд": `http://push:9099/bitrix/pub/`.

## Полная настройка с nginx

Если Битрикс24 работает за nginx, нужно настроить прокси для WebSocket/Long-polling. Проект тестировался на сборке [aclips/bitrix24-docker](https://github.com/aclips/bitrix24-docker), но подойдёт и другое окружение.

### 1. Настроить nginx

В `http`-секцию конфигурационного файла nginx добавить:

```nginx
upstream push-upstream {
    server push:9099;
}

map $http_upgrade $connection_upgrade {
  default upgrade;
  '' 'close';
}

map $http_upgrade $replace_upgrade {
  default $http_upgrade;
  ''      "websocket";
}
```

В `server`-секцию добавить:

```nginx
location ~* ^/bitrix/subws/ {
    proxy_pass http://push-upstream;
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

### 2. Добавить сервис в docker-compose

```yaml
  push:
    build:
      context: ./containers/push
      dockerfile: ./Dockerfile
    container_name: ${PROJECT_PREFIX}_push
```

### 3. Подготовить файлы

В директории `./containers/push` разместить два файла:

**Dockerfile:**

```dockerfile
FROM gromdron/bitrix-push-workspace:v2.0

WORKDIR /opt/push-server

EXPOSE 9099

COPY push_config.toml .

ENV CONFIG_FILE=/opt/push-server/push_config.toml

CMD ["push-server"]
```

**push_config.toml** — скопировать из корня репозитория и настроить под проект.

## Конфигурация

Файл `push_config.toml` — основной конфиг. Параметры:

| Параметр | По умолчанию | Описание |
|----------|-------------|----------|
| `general.port` | 9099 | Порт сервера |
| `history.max_length` | 5000 | Максимум сообщений в истории |
| `history.max_ttl` | 600 | Максимальное время жизни сообщений (сек) |
| `history.default_ttl` | 30 | TTL по умолчанию (сек) |
| `security.enabled` | false | Проверка HMAC-подписи каналов |
| `security.key` | — | Ключ подписи |
| `log.level` | trace | Уровень логирования |
| `long_poll.timeout_secs` | 30 | Таймаут long-poll запросов (сек) |

Параметры можно переопределить через переменные окружения с префиксом `PUSH_`, например: `PUSH_GENERAL_PORT=8080`.

## Сборка из исходников

Для самостоятельной сборки потребуется:

- Rust 1.87+
- protobuf-compiler

```bash
# Установить зависимости (Ubuntu/Debian)
sudo apt install -y protobuf-compiler
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.87

# Клонировать и собрать
git clone git@github.com:gromdron/bitrix-push-workspace.git
cd bitrix-push-workspace
cargo build --release
```

Готовый бинарник: `target/release/bitrix-push-workspace`.
