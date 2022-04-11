Применяется для логической группировки данных. Также является средством
разграничения [прав доступа](/components/relay/configuration/permissions/).

### Данные объекта

#### Идентификатор (ID)

Уникальный идентификатор служит для адресации объекта в командах. Бывает двух типов - id объекта созданного игроком, и
id объекта созданного комнатой.

#### Шаблон (Template)

Описывает тип объекта — например танк, мина и так далее. Позволяет клиенту понять что конкретно надо создавать, также
для разных шаблонов можно настраивать права доступа, трассировку команд и так далее.

#### Владелец (Owner)

Поддерживаются два типа владельцев — игрок и комната. Объекты, созданные игроками, удаляются при удалении игрока из
комнаты (но не при DetachFromRoom)

#### Группы (Groups)

Список [групп доступа](/components/relay/configuration/permissions/)

#### Данные объекта

Данные разных типов.

## Создание

### Создание объекта от имени игрока

```csharp    
var builder = client.NewObjectBuilder(template, accessGroup, keepAfterOwnerExit);
builder.SetDouble(1, 100.0);
builder.SetLong(2, 100500);
builder.SetStructure(1, new SomeStructure());
var cheetahObject = builder.Build();
```

- keepAfterOwnerExit - не удалять объект при выходе пользователя из игры; 

Особенности:

- Порядок и гарантии доставки команд зависят от типа канала. Для ненадежных каналов может прийти только часть команд.
- Сервер не сохраняет порядок установки данных для игрового объекта.
- Сервер не посылает данную команду обратно, исключение только для объектов созданных от имени данного клиента на
  сервере.

### Создание объекта от имени комнаты

Возможно только в [конфигурации комнаты.](/components/relay/configuration/room/)

### Обработчик загружаемых на клиент объектов

```csharp
  
  // создаем один раз как переменную класса
  CreatedObjectByTemplateIncomeCommands listener = new CreatedObjectByTemplateIncomeCommands(client, template);
  
  void Update() {
    foreach (var objectConstructor in listener.GetStream())
    {
      var obj = objectConstructor.cheetahObject;
      var damage = objectConstructor.GetDouble(100);
      if (objectConstructor.TryGetDouble(100, var out damage)) {
      
      }
      
    }
  }
```

## Удаление

### Удаление объекта

```csharp
cheetahObject.Delete();
```
- Команда удаления не отсылается обратно пользователю, который удалил объект.

### Подписка на удаление объекта

```csharp
DeletedObjectByTemplateIncomeCommands listener = new DeletedObjectByTemplateIncomeCommands(client, template);

void Update() {
  var deleted = listener.GetStream();
  for (var i = 0; i < deleted.Count; i++)
  {
    ref var obj = ref deleted.GetItem(i);
  }
}        
```