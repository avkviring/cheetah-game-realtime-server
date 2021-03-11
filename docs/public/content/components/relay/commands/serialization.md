В некоторых командах (структуры, события) используется массив байт для представления информации. Существует два способа
формирования таких данных — непосредственно написать код, преобразующий данные в бинарный вид, или воспользоваться
библиотекой MessagePack.

### Пример использования

```csharp

[MessagePackObject]
public class SomeStructure {
    [Key(0)] public long Age { get; set; }    
}

var codecRegistry = new CodecRegistry();
codecRegistry.RegisterStructure(FieldId, new MessagePackCodec<SomeStructure>());
var originalMessage = new SomeStructure {Age = 100500};

var buffer = new CheetahBuffer();
codecRegistry.EncodeStructure(FieldId, originalMessage, ref buffer);
var message = (SomeStructure) codecRegistry.DecodeStructure(FieldId, ref buffer);
```

### Документация

[Официальный сайт.](https://github.com/neuecc/MessagePack-CSharp)

Раздел о подключении можно пропустить, так как данная библиотека уже подключена к платформе.

Следует обратить внимание на то, что для Android & iOS требуется сгенерить кодеки до этапа сборки.