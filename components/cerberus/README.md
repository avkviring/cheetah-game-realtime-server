# Cerberus
Сервис для работы с JWT токенами.
- создание токенов
- обновление токенов


## Состав

### server/Service
Состоит из двух gRPC серверов:
- internal (порт 5002) - генерация пары session/refresh и привязка его к пользователю 
- external (порт 5002) - обновление пары session/refresh по refresh токену

### server/Library
Библиотека для валидации токена. Используется в других микросервисах для получения идентификатора
пользователя из токена, а также для проверки валидности токена.




## Запуск
### Генерация ключей
```shell
openssl ecparam -name prime256v1 -genkey -out private.pem
openssl pkcs8  -topk8 -nocrypt -in private.pem -out private-pkcs8.pem 
openssl ec -in private.pem -pubout -out public.pem
cat private-pkcs8.pem | base64 
cat public.pem | base64
```

### Настройка
Конфигурация передается через переменные окружения.

- JWT_PUBLIC_KEY - base64 decoded 
- JWT_PRIVATE_KEY - base64 decoded
- REDIS_HOST
- REDIS_PORT

### Docker image
```
docker run docker.registry.cheetah.games/cheetah.games/cerberus:version
```

# Использование

- Auth сервис создает с помощью interal сerberus пару session/refresh токен и отсылает их клиенту;
- Клиент использует session токен при запросах на микросервисы;
- Как только время жизни session токена истечет, то клиент должен обновить session токен с помощью refresh токена;
- Если время жизни refresh токена истекло, то клиент заново должен пройти процедуру авторизации;

# Время жизни токенов

Задаются константами в файле. Так как время жизни записывается в токене, то новые значения будут
применены только для новых токенов.
Так как session токены нельзя отозвать, то время его действия не должно превышать пары часов, в идеале 10-15 минут.
Refresh токены отозвать можно, однако на данный момент такой команды у сервиса нет.

[cerberus.rs](server/Service/src/cerberus.rs)
