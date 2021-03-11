Определяет список объектов комнаты и список ожидаемых пользователей с пользовательскими объектами. 
Обязательно для запуска без MM сервера.

``` shell
server --room room1.yaml --room room2.yaml ...
```

### Формат файла конфигурации комнаты

``` yaml
id: 100
# автосоздание пользователя после выхода
auto_create_user: true 
# список ожидаемых пользователей
users:
  - id: 1 # id пользователя, должен быть уникальным в рамках комнаты
    private_key: [ 1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2,3,4,5,6,7,8,9,0,1,2 ]
    access_groups: 15
    # данные объекты будут созданы от имени пользователя при входе пользователя в комнату
    objects:
      - id: 100 # допустимый диапазон - 0 .. 512 - GameObjectId::CLIENT_OBJECT_ID_OFFSET
        template: 4
        access_groups: 15
        fields:
          longs:
            5: 100
            15: 200
          floats:
            3: 5.5
            7: 9.9
          structures:
            10:
              name: alex
# объекты комнаты
objects:
  - id: 5
    template: 5
    access_groups: 0
    fields:
      longs: { }
      floats: { }
      structures: { }
```
