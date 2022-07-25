# Проверка совместимости клиента и сервера

Пакет **games.cheetah.system.compatibility**.

### Проверка совместимости

```csharp
var connectorFactory = new ConnectorFactory();
await connectorFactory.Connect();
var clusterConnector = connectorFactory.ClusterConnector;
var checker = new CompatibilityChecker(clusterConnector);
var status = await checker.Check("0.0.1");
```

### Статусы

```protobuf
 enum Status {
  /**
    Версия клиента поддерживается, в обновлении нет необходимости
   */
  Supported = 0;
  /**
     Планируется окончания поддержки - за 4-24 часа до окончания
   */
  PlanningUnsupportedAfterSomeHours = 1;
  /**
    Планируется окончания поддержки - за 0-4 часа до окончания
   */
  PlanningUnsupportedSoon = 2;
  /**
    Данная версия клиента не поддерживается, если в это время клиент в битве - необходимо обновить клиент после
    окончания битвы, если клиент только что запущен или в процессе запуска - необходимо принудительное обновление
   */
  Unsupported = 3;
}
```

### Конфигурирование сервера

Файл Editor/Cheetah/Production/system-compatibility/versions.yaml

```yaml
versions:
  - version: 0.0.1
    expiration: 2021-12-10 15:17
  - version: 0.0.2
    expiration: never
```