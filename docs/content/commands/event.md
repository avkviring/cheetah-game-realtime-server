События — набор бинарных данных с идентификатором поля. События не сохраняются на объекте, а сразу рассылаются по
клиентам.

### Отправка на сервер

Событие рассылается всем пользователя, на которых загружен объект.

```csharp
cheetahObject.SendEvent<T>(ushort eventId, ref T item)

```

Событие отправляется конкретному пользователю если на него загружен объект.

```csharp
cheetahObject.SendEvent<T>(ushort eventId, uint targetUser, ref T item)
```

### Обработка событий с сервера

События по определенному полю.

```csharp
EventIncomeByFieldCommandCollector listener = new EventIncomeByFieldCommandCollector<SomeStructure>(client, fieldId);
void Update() {
  var stream = listener.GetStream();
  for (var i = 0; i < stream.Count; i++)
  {
    ref var item = ref stream.GetItem(i);
    var obj = item.cheetahObject;
    ref var value = ref item.value;
  }
}
```

События по определенному полю и игровому объекту.

```csharp
EventIncomeByObjectCommandCollector listener = new EventIncomeByObjectCommandCollector<SomeStructure>(client, objectId, fieldId);
void Update() {
  var stream = listener.GetStream();
  for (var i = 0; i < stream.Count; i++)
  {
    ref var item = ref stream.GetItem(i);
    ref var value = ref item.value;
  }
}
```


