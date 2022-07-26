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
