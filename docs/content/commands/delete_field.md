Удаление поля с объекта

- рассылается уведомление об удалении всем существующим клиентам
- для новых клиентов данное поле не будет загружено при загрузке объекта
- если поле не существует - команда выполнится так как будто поле есть на объекте

### Удаление поля

```csharp
cheetahObject.DeleteField(ushort fieldId, FieldType fieldType)
```


### Обработка изменений с сервера

Изменения для пары поле плюс объект

```csharp
DeleteFieldIncomeByObjectCommandCollector collector = new DeleteFieldIncomeByObjectCommandCollector(client, objectId, field);

void Update() {
 var stream = collector.GetStream();
 for (var i = 0; i < stream.Count; i++)
 {
     ref var item = ref stream.GetItem(i);        
     var fieldType = item.value;     
 }
}
```

Изменения для пары поле плюс шаблон
```csharp
 DeleteFieldIncomeByObjectCommandCollector collector = new DeletedFieldByTemplateIncomeCommands(client, template);
 
 void Update() {
    var stream = collector.GetStream();
    for (var i = 0; i < stream.Count; i++)
    {
         ref var item = ref stream.GetItem(i);        
        var fieldType = item.fieldType;
        var fieldId = item.fieldId
        var cheetahObject = fieldType.cheetahObject
    }
   }
```



