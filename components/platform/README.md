# Точка входа для работы с сервисами платформы

- создает и кеширует grpc channel
- возможно, несет на себе другие функции (пока не понятно какие)

```csharp
var platform = new Cheetah.Platform("https://some-service-url:8080/");
var player = new Cheetah.Player(platform, new Cheetah.Auth.Cookie());
player.login();
```

