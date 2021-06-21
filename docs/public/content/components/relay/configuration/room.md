### Конфигурация комнаты

Каталог - Assets/Editor/Cheetah/Relay/Templates

Формат имени файла - \*.yml

``` yaml
# описание объектов комнаты
objects:
  - id: 5
    template: 5
    access_groups: 0
    fields:
      longs: { }
      floats: { }
      structures: { }
# описание прав доступа      
permissions:
    ...      
```

**Подробная информация о назначении полей**

- *objects* - [Объектная модель](../basics/object.md).
- *permissions* - [Правила описания прав доступа](permissions.md).
