using System;

namespace Cheetah.Matches.Relay.Codec {
    /// <summary>
    /// Для структуры, помеченной этим полем, необходимо автоматически создать кодек
    /// </summary>
    [AttributeUsage(AttributeTargets.Struct)]
    public class GenerateCodec : Attribute { }

    /// <summary>
    /// Поле следует декодировать переменным числом байт
    /// </summary>
    [AttributeUsage(AttributeTargets.Field)]
    public class VariableSizeCodec : Attribute { }

    [AttributeUsage(AttributeTargets.Field)]
    public class ArraySizeField : Attribute {
        public ArraySizeField(string field) {
            Field = field;
        }

        public readonly string Field;
    }
}