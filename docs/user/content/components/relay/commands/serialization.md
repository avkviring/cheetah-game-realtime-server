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

Для всех структур отмеченных аннотацией [GenerateCodec] производится генерация кодеков. Для генерации кодеков необходимо
вызвать команду Windows/Cheetah/Generate codecs. Такие кодеки не надо отдельно регистрировать в реестре.

### Глобальные кодеки

Кодеки, зарегистрированные таким способом будут автоматически добавлены во все новые реестры.

```csharp
CodecRegistryBuilder.RegisterDefault(factory=>new DropMineEventCodec());
```

### Реализация кодека

Применяется в случаях - если сгенерированный кодек не подходит или нужен кодек для класса из сторонней библиотеки.

```csharp
public class DropMineEventCodec:Codec<Shared.Types.DropMineEvent>
{
	public void Decode(ref CheetahBuffer buffer, ref Shared.Types.DropMineEvent dest)
	{
		dest.MineId = IntFormatter.Instance.Read(ref buffer);
	}

	public void  Encode(ref Shared.Types.DropMineEvent source, ref CheetahBuffer buffer)
	{
		IntFormatter.Instance.Write(source.MineId,ref buffer);
	}


	[RuntimeInitializeOnLoadMethod(RuntimeInitializeLoadType.SubsystemRegistration)]
	private static void OnRuntimeMethodLoad()
	{
		CodecRegistryBuilder.RegisterDefault(factory=>new DropMineEventCodec());
	}

}
```
Для работы с бинарными данными существует несколько вспомогательных классов:

- ByteFormatter
- DoubleFormatter
- FloatFormatter
- BoolFormatter
- IntFormatter
- LongFormatter
- ShortFormatter
- StringFormatter
- UIntFormatter
- ULongFormatter
- UShortFormatter
- VariableSizeIntFormatter
- VariableSizeLongFormatter
- VariableSizeUIntFormatter
- VariableSizeULongFormatter


