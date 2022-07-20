using Cheetah.Matches.Relay.Codec;
using JetBrains.Annotations;

namespace Cheetah.Matches.Relay.Editor.Generator.Fields
{
    public class EnumFieldGenerator : FieldCodecGenerator
    {
        private readonly string formatterName;
        private readonly FieldInfoAccessor field;

        [CanBeNull]
        public static FieldCodecGenerator Create(Formatters formatters, FieldInfoAccessor fieldInfo)
        {
            return fieldInfo.FieldType.IsEnum ? new EnumFieldGenerator(formatters.GetFormatterInstanceName(typeof(byte)), fieldInfo) : null;
        }

        private EnumFieldGenerator(string formatterName, FieldInfoAccessor field)
        {
            this.formatterName = formatterName;
            this.field = field;
        }

        public string GenerateEncode()
        {
            return $"{formatterName}.{nameof(Formatter<bool>.Write)}((byte)source.{field.Name},ref buffer);";
        }

        public string GenerateDecode()
        {
            return $"dest.{field.Name} = ({Utils.GetFullName(field.FieldType)}){formatterName}.{nameof(Formatter<bool>.Read)}(ref buffer);";
        }
    }
}