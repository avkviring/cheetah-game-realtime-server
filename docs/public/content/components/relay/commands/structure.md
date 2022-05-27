Структура — набор бинарных данных с идентификатором поля. Структуры для объекта сохраняются на сервере и загружаются при
загрузке объекта на клиента. Также установка новой структуры или обновление существующей приводит к изменению структуры
на всех клиентах, на которых загружен игровой объект

### Изменение структуры

```csharp
cheetahObject.SetStructure<T>(ushort fieldId, ref T item)
```

### Обработка изменений структуры с сервера

Изменения для определенного поля.

```csharp
StructureIncomeByFieldCommandCollector listener = new StructureIncomeByFieldCommandCollector<SomeStructure>(client,  fieldId);

void Update() {
 var stream = listener.GetStream();
 for (var i = 0; i < stream.Count; i++)
 {
     ref var item = ref stream.GetItem(i);
     ref var obj = ref item.cheetahObject;
     ref var value = ref item.value;
 }
}
```

Изменения для пары поле плюс объект

```csharp
StructureIncomeByObjectCommandCollector listener = new StructureIncomeByObjectCommandCollector<SomeStructure>(client, objectId, fieldId);

void Update() {
 var stream = listener.GetStream();
 for (var i = 0; i < stream.Count; i++)
 {
     ref var item = ref stream.GetItem(i);
     ref var value = ref item.value;
 }
}
```
