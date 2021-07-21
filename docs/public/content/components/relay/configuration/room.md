### Конфигурация комнаты

Каталог - Assets/Editor/Cheetah/Relay/Templates

Формат имени файла - \*.yml

``` yaml
# описание предопределенных игровых объектов комнаты
objects:
  - id: 555
    template: 1
    groups: 4857
    fields:
      longs:
        10: 100100
      floats:
        5: 3.14
        10: 2.68
      structures:
        5:
          uid: arts80
          rank: 100
        10:
          - 15
          - 26
# описание прав доступа к объектам 
permissions:
  templates:
    - template: 1
      rules:
        - groups: 12495
          permission: deny
      fields:
        - id: 100
          type: long
          rules:
            - groups: 5677
              permission: ro
```

**Подробная информация о назначении полей**

- *objects* - [Объектная модель](../basics/object.md).
- *permissions* - [Правила описания прав доступа](permissions.md).
