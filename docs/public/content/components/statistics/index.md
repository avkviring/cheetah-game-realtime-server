# Сбор событий и логов с клиента

Пакет **games.cheetah.statistics.events**.

### Отправка событий

```csharp
var sender = new EventsSender(clusterConnector);
sender.SendEvent("some event");
sender.SendEvent("play", new Dictionary<string, string>()
{
    ["user"] = "Петя"
});
```

### Отправка логов

```csharp
var sender = new LogsSender(clusterConnector);
// отправка логов
sender.SendLog(LogType.Error, "it is error", "some stack trace");
// также перехватываются все логи из Unity
Debug.LogWarning("it is warning");
```

### Просмотр логов & событий

Для просмотра и анализа используется любой WEB UI для Loki.

Метки для поиска:

- type = event | log
- source = client
- namespace - kubernetes namespace




