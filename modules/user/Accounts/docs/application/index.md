# Введение

## Настройка среды разработки

Установить пакет **games.cheetah.accounts** в Package Manager.

### Схема использования

- получаем токены используя либо внешнюю систему аутентификации, либо временную регистрацию;
- сохраняем токены на клиенте и пользуемся ими для авторизации пользователя на сервере;
- в такой схеме нет необходимости пользоваться внешний аутентификацией каждый раз, достаточно использовать сохраненные
  токены;

<!- Оригинал - Assets/Scripts/DocumentationExample.cs функция AuthExample->

```csharp
// соединяемся с кластером платформы
var clusterConnector = new ClusterConnector("some-cluster", 5000, true);
        
// попытка входа с сохраненными токенами
var storedTokenUserAuthenticator = new StoredTokenUserAuthenticator();
var user = await storedTokenUserAuthenticator.Login(clusterConnector);
if (user == null)
{
  // если сохраненных токенов нет
  // то запускаем внешнию авторизацию, например Cookie
  var cookieAuthenticator = new CookieAuthenticator(clusterConnector, "user1");
  cookieAuthenticator.RemoveLocalCookie();
  var authenticationResult = await cookieAuthenticator.LoginOrRegister();
  user = authenticationResult.User;

  // сохраняем токены для использования после перезапуска приложения
  storedTokenUserAuthenticator.Store(user);
}
```
