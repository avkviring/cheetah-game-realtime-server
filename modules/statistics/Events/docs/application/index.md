# Сбор событий и логов с клиента

Пакет **games.cheetah.statistics.events**.

## Отправка событий

```csharp
var session = new StatisticsSession(clusterConnector);
var eventsSender = new EventsSender(session);
eventsSender.Send("test");
eventsSender.Send("play", new Dictionary<string, string>()
{
    ["user"] = "Петя"
});
```

## Отправка логов

```csharp
var session = new StatisticsSession(clusterConnector);
// перехватчик логов из Unity
new UnityDebugLogSender(session);
Debug.LogError("it is error");
Debug.LogException(new NullReferenceException());
```

## Общие метки для всех событий

- type = event|log
- source = client|server
- namespace - kubernetes namespace
- session

## Сессия

Сессия - это время от запуска приложения до его закрытия. 
У сессии есть уникальный идентификатор, который сохраняется в метке session.

### Событие открытия сессии
Данное событие посылается один раз при создании клиента.
Метка для поиска событий открытия сессий - *open_session=true*

Сохраняемая информация:

```
["device_id"] = SystemInfo.deviceUniqueIdentifier,
["device_name"] = SystemInfo.deviceName,
["device_model"] = SystemInfo.deviceModel,
["processor_type"] = SystemInfo.processorType,
["processor_count"] = SystemInfo.processorCount.ToString(),
["processor_frequency"] = SystemInfo.processorFrequency.ToString(),
["battery_status"] = SystemInfo.batteryStatus.ToString(),
["battery_level"] = SystemInfo.batteryLevel.ToString(),
["graphics_device_model"] = SystemInfo.graphicsDeviceName,
["graphics_device_vendor"] = SystemInfo.graphicsDeviceVendor,
["graphics_device_version"] = SystemInfo.graphicsDeviceVersion,
```

### Событие закрытие сессии

Метка для поиска событий открытия сессий - *close_session=true*

## Метки для логов

- level - error|exception
- line - номер строки из stacktrace(при наличии)
- file - имя файла из stacktrace (при наличии)
- count - количество повторов одного и того же сообщения об ошибке, рассылается первые 5 сообщений, потом раз в минуту.
