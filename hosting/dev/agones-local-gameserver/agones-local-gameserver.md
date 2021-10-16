# Локальный Запуск GameServer и Agones sidecar без kubernetes

Этот гайд поможет настроить локальное окружение для разработки и отладки работы с Agones SDK внутри Relay сервера.

[Официальный гайд](https://agones.dev/site/docs/guides/client-sdks/local/)

## Запуск

1. Загрузить `agonessdk-server` по [ссылке в начале гайда](https://agones.dev/site/docs/guides/client-sdks/local/#local-development)
2. Запустить `agonessdk-server`
    ```bash
    ~/Downloads/agonessdk-server-1.18.0/sdk-server.darwin.amd64 --local
    ```
    Если сервер не запускается с ошибкой _“sdk-server.darwin.amd64” cannot be opened because the developer cannot be verified._,
    зайдите в **System Preferences -> Security and Privacy -> General** и нажмите **Allow Anyway**.
3. Запустите Relay сервер в другой консоли
    ```bash
    cd platform/server/matches/Relay/Server
    ENABLE_AGONES=true cargo run
    ```

В логах Relay сервера должно быть:
```
2021-10-16T19:11:30.531Z INFO  cheetah_matches_relay::agones      > Agones: Starting
2021-10-16T19:11:30.573Z INFO  cheetah_matches_relay::agones      > Agones: Connected to SDK
2021-10-16T19:11:30.574Z INFO  cheetah_matches_relay::agones      > Agones: invoked sdk.mark_ready
2021-10-16T19:11:30.574Z INFO  cheetah_matches_relay::agones      > Agones: invoked health
2021-10-16T19:11:32.575Z INFO  cheetah_matches_relay::agones      > Agones: invoked health
...
```
А в логах `agonessdk-server`:
```
{"message":"Getting GameServer details","severity":"info","source":"*sdkserver.LocalSDKServer","time":"2021-10-16T22:11:30.56839+03:00"}
{"message":"Ready request has been received!","severity":"info","source":"*sdkserver.LocalSDKServer","time":"2021-10-16T22:11:30.573694+03:00"}
{"message":"Gameserver update received","severity":"info","source":"*sdkserver.LocalSDKServer","time":"2021-10-16T22:11:30.573813+03:00"}
{"message":"Health Ping Received!","severity":"info","source":"*sdkserver.LocalSDKServer","time":"2021-10-16T22:11:30.574948+03:00"}
...
```

## Имитация отправки команд в GameServer из Agones

`agonessdk-server` можно использовать для тестирования поведения Relay сервера при получении команд от Agones.

shutdown:
```bash
curl -X POST "http://localhost:9358/shutdown" -H "accept: application/json" -H "Content-Type: application/json" -d "{}"
```

allocate:
```bash
curl -X POST "http://localhost:9358/allocate" -H "accept: application/json" -H "Content-Type: application/json" -d "{}"
```

Весь список поддерживаемых команд в [официальном гайдe](https://agones.dev/site/docs/guides/client-sdks/local/#changing-state-of-a-local-gameserver).
