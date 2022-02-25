Для отсылки сообщений и структур на сервер используются кодеки, основная задача которых получить из структуры массив
байт и обратно.

### Реестр кодеков

Для подключения к серверу необходимо в метод создания клиента передать реестр кодеков.

```csharp
var coderRegistryBuilder = new CodecRegistryBuilder();
// региструем кодек
coderRegistryBuilder.Register<SomeStructure>(new SomeStructureCodec());
// создаем реестр
var codecRegistry = coderRegistryBuilder.Build();
```

### Генерация кодеков

Для всех структур отмеченных аннотацией [GenerateCodec] производится генерация кодеков. Для генерации кодеков 
необходимо вызвать команду Windows/Cheetah/Generate codecs. Такие кодеки не надо отдельно регистрировать в реестре.


### Глобальные кодеки

Кодеки, зарегистрированные таким способом будут автоматически добавлены во все новые реестры.

```csharp
CodecRegistryBuilder.RegisterDefault(factory=>new DropMineEventCodec());
```


