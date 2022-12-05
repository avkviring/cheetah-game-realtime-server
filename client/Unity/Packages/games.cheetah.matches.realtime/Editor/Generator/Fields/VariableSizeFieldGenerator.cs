using System;
using System.Runtime.CompilerServices;
using Cheetah.Matches.Realtime.Codec;
using JetBrains.Annotations;

namespace Cheetah.Matches.Realtime.Editor.Generator.Fields
{
    public class VariableSizeFieldGenerator : FieldCodecGenerator
    {
        private readonly FieldInfoAccessor field;
        private readonly string formatterName;

        [CanBeNull]
        public static VariableSizeFieldGenerator Create(Formatters formatters, FieldInfoAccessor field)
        {
            if (field.GetCustomAttribute<FixedBufferAttribute>() != null)
            {
                return null;
            }

            if (field.FieldType.IsArray)
            {
                return null;
            }

            var attribute = field.GetCustomAttribute<VariableSizeCodec>();
            return attribute == null ? null : new VariableSizeFieldGenerator(formatters, field);
        }

        private VariableSizeFieldGenerator(Formatters formatters, FieldInfoAccessor field)
        {
            this.field = field;
            formatterName = formatters.GetVariableSizeFormatterInstanceName(field.FieldType) ??
                            throw new Exception(nameof(VariableSizeCodec) + " don't support type " + field.FieldType.Name);
        }

        public string GenerateEncode()
        {
            return $"{formatterName}.{nameof(Formatter<bool>.Write)}(source.{field.Name},ref buffer);";
        }

        public string GenerateDecode()
        {
            return $"dest.{field.Name} = {formatterName}.{nameof(Formatter<bool>.Read)}(ref buffer);";
        }
    }
}