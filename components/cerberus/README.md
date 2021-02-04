# cerberus
Сервис для работы с JWT токенами.

## server/Service
Микросервис для выдачи и обновления токенов.

### Настройка

**Необходимо сгенерировать ключи для ECC**

```shell
openssl ecparam -name prime256v1 -genkey -out private.pem
openssl pkcs8  -topk8 -nocrypt -in private.pem -out private-pkcs8.pem 
openssl ec -in private.pem -pubout -out public.pem
cat private-pkcs8.pem 
cat public.pem  
```
## server/Library
Библиотека для валидации токена. Используется в других микросервисах для получения идентификатора 
пользователя из токена, а также для проверки валидности токена.