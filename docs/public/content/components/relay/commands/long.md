На каждом объекте можно хранить целые числа (signed 64-bit), адресуемых по идентификатору поля. Все сохраненные поля
загрузятся клиенту при загрузке объекта. Также любое обновление поля на одном клиенте будет разослано другим клиентам.

### Установка нового значения

```csharp
cheetahObject.SetLong(ushort fieldId, long value)
```

### Инкремент/декремент значения

```csharp
cheetahObject.IncrementLong(ushort fieldId, long increment)
```

### CompareAndSet

```csharp
cheetahObject.CompareAndSet(ushort fieldId, long currentValue, long newValue, long resetValue);
```

Применяется для определения первого клиента при выполнении одновременных действия, например, с помощью данного метода
можно определить кто первый взял бонус.

- currentValue - необходимое значение в поле для выполнения операции;
- newValue - новое значение поля, если текущее значение совпадает с currentValue;
- resetValue - значение поля, при выходе игрока из битвы, если была осуществлена операция установки newValue.

#### Пример использования

Допустим нам надо определить кто первый бонус, для этого все клиенты посылаются команду CompareAndSet(objectId, 0,
currentUserId, 0). Выполниться только первая команда, так как currentValue после нее будет уже не ноль и другие команды
не смогут переписать значение поля.

### Обработка изменения

Изменения для определенного поля.

```csharp
LongIncomeByFieldCommandCollector listener = new LongIncomeByFieldCommandCollector(client, field);

void Update() {
 var stream = listener.GetStream();
 for (var i = 0; i < stream.Count; i++)
 {
     ref var item = ref stream.GetItem(i);
     var obj = item.cheetahObject;
     var value = item.value;
     var creator = item.commandCreator;
 }
}
```

Изменения для пары поле плюс объект

```csharp
LongIncomeByObjectCommandCollector listener = new LongIncomeByObjectCommandCollector(client, objectId, field);

void Update() {
 var stream = listener.GetStream();
 for (var i = 0; i < stream.Count; i++)
 {
     ref var item = ref stream.GetItem(i);        
     var value = item.value;
     var creator = item.commandCreator;
 }
}
```

