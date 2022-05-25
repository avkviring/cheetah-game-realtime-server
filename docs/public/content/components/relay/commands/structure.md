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

### CompareAndSet

Применяется для определения первого клиента при выполнении одновременных действия
C помощью этого метода, например, можно определить, кто первый взял бонус.

```csharp
cheetahObject.CompareAndSetStructure(ushort fieldId, ref T current, ref T new);
```

- currentValue — необходимое значение в поле для выполнения операции;
- newValue — новое значение поля, если текущее значение совпадает с currentValue;

```csharp
cheetahObject.CompareAndSetStructureWithReset(ushort fieldId, ref T current, ref T new, ref T reset);
```

- resetValue — значение поля при выходе игрока из битвы, если была осуществлена операция установки newValue.
- currentValue — необходимое значение в поле для выполнения операции;
- newValue — новое значение поля, если текущее значение совпадает с currentValue;
- resetValue — значение поля, при выходе игрока из битвы, если была осуществлена операция установки newValue.

#### Пример использования

Допустим нам надо определить кто первый взял бонус. Для этого все клиенты посылают команду CompareAndSetStructure
с одинаковым значением currentValue.
Выполнится только первая обработанная команда, так как currentValue после нее будет уже отличаться от исходного,
и другие команды не смогут переписать значение поля.
