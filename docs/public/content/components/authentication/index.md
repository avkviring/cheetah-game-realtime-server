# Введение

Для работы с сервисами с клиента используется [JWT](https://jwt.io) токен, который содержит id игрока.

## Схема работы

- получаем JWT токены используя либо внешнюю систему аутентификации, либо временную регистрацию;
- сохраняем JWT токены на клиенте и пользуемся ими для авторизации пользователя на сервере;
- в такой схеме нет необходимости пользоваться внешний аутентификацией каждый раз, достаточно использовать сохраненные
  токены;

## Пример кода авторизации

```csharp
  // соединение с серверной платформой
  var Connector connector = new Connector("host:port");
  // попытка входа с сохраненными токенами
  var storedAuthenticator = new StoredPlayerAuthenticator();
  var player = await storedAuthenticator.Login(connector);
  if (player != null)
  {   
     return player;
  }
  else
  {
    #if UNITY_ANDROID
      // если сохраненных токенов нет - то запускаем внешнию авторизацию
      var androidAuthenticator = new AndroidAuthenticator(androidWebClientId);
      var result = await androidAuthenticator.LoginOrRegister(connector);
      var player = result.Player;
      // сохраняем токены для использования после перезапуска приложения
      storedAuthenticator.Store(player);
      return player;
    #endif
  }
```

Класс StoredPlayerAuthenticator используется для сохранения токенов авторизации между запусками приложения. Если его не
использовать — то необходимо будет проводить Android авторизацию каждый раз при запуске.


