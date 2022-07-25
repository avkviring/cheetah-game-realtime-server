Для некоторых типов данных доступен метод CompareAndSet.

К таким типам на текущий момент относятся:

* Целые числа,
* Структуры

Этот метод можно применять для определения первого клиента при выполнении одновременных действия.
Например, данный метод позволяет определить, кто первый взял бонус.

## CompareAndSetLong

```csharp
cheetahObject.CompareAndSetLong(ushort fieldId, long currentValue, long newValue);
```

* currentValue - необходимое значение в поле для выполнения операции;
* newValue - новое значение поля, если текущее значение совпадает с currentValue;

```csharp
cheetahObject.CompareAndSetLongWithReset(ushort fieldId, long currentValue, long newValue, long resetValue);
```

* resetValue — значение поля при выходе игрока из битвы, если была осуществлена операция установки newValue.
* currentValue — необходимое значение в поле для выполнения операции;
* newValue — новое значение поля, если текущее значение совпадает с currentValue;
* resetValue — значение поля, при выходе игрока из битвы, если была осуществлена операция установки newValue.

## CompareAndSetStructure

```csharp
cheetahObject.CompareAndSetStructure(ushort fieldId, ref T current, ref T new);
```

* currentValue — необходимое значение в поле для выполнения операции;
* newValue — новое значение поля, если текущее значение совпадает с currentValue;

```csharp
cheetahObject.CompareAndSetStructureWithReset(ushort fieldId, ref T current, ref T new, ref T reset);
```

* resetValue — значение поля при выходе игрока из битвы, если была осуществлена операция установки newValue.
* currentValue — необходимое значение в поле для выполнения операции;
* newValue — новое значение поля, если текущее значение совпадает с currentValue;
* resetValue — значение поля, при выходе игрока из битвы, если была осуществлена операция установки newValue.

### Пример использования

Допустим нам надо определить кто первый взял бонус. Для этого все клиенты посылают команду CompareAndSetLong
с одинаковым значением currentValue.
Выполнится только первая обработанная команда, так как currentValue после нее будет уже отличаться от исходного,
и другие команды не смогут переписать значение поля.
