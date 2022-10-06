# Интеграционное тестирование

Для интеграционного тестирования есть возможность запустить встроенный Realtime сервер из C# кода.

## Настройка среды разработки

Установить пакет **games.cheetah.matches.realtime.embedded-server** в Package Manager.

## Пример использования

```csharp
    // если нам необходимо посмотреть логи с сервера
    API.EmbeddedServer.InitLogger(CheetahLogLevel.Warn);

    var server = new API.EmbeddedServer();
    var room = server.CreateRoom();
    var member = room.CreateMember(0b000111);

    // сервер создан, теперь к нему можно подключаться обычным способом
    var client = new CheetahClient(
               server.GetGameHost(),
               server.GetGamePort(),
               member.GetId(),
               room.GetId(),
               member.GetPrivateKey(),
               new CodecRegistryBuilder().Build());
               
    ...
    
    // после теста для освобождения ресурсов
    server.Destroy();    
    
    // показываем собранные логи с сервера
    API.EmbeddedServer.ShowCurrentLogs();
```

