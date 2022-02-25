На каждом объекте можно хранить вещественные числа (double), адресуемых по идентификатору поля. Все сохраненные поля
загрузятся клиенту при загрузке объекта. Также любое обновление поля на одном клиенте будет разослано другим клиентам.

### Установка нового значения

```csharp
cheetahObject.SetDouble(ushort fieldId, double value)
```

### Инкремент/декремент значения

```csharp
cheetahObject.IncrementDouble(ushort fieldId, double increment)
```

### Обработка изменений с сервера

Изменения для определенного поля.

```csharp
DoubleIncomeByFieldCommandCollector listener = new DoubleIncomeByFieldCommandCollector(client, field);

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
DoubleIncomeByObjectCommandCollector listener = new DoubleIncomeByObjectCommandCollector(client, objectId, field);

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


