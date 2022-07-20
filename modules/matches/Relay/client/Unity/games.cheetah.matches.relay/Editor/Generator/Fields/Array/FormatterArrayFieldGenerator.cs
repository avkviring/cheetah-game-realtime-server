using System.Collections.Generic;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Editor.Generator.Fields.Array.Exceptions;
using JetBrains.Annotations;

namespace Cheetah.Matches.Relay.Editor.Generator.Fields.Array
{
    public class FormatterArrayFieldGenerator : FieldCodecGenerator
    {
        private readonly FieldInfoAccessor field;
        private readonly ArraySizeField sizeFieldAttribute;
        private readonly string formatterName;

        [CanBeNull]
        public static FormatterArrayFieldGenerator Create(Formatters formatters, FieldInfoAccessor field, HashSet<string> processedFields,
            HashSet<string> allFields)
        {
            if (!field.FieldType.IsArray)
            {
                return null;
            }

            string formatterName;
            var variableSizeCodecAttribute = field.GetCustomAttribute<VariableSizeCodec>();
            var elementType = field.FieldType.GetElementType();
            if (variableSizeCodecAttribute == null)
            {
                formatterName = formatters.GetFormatterInstanceName(elementType);
            }
            else
            {
                formatterName = formatters.GetVariableSizeFormatterInstanceName(elementType);
                if (formatterName == null)
                {
                    throw new VariableSizeTypeNotSupportedException(elementType);
                }
            }

            return formatterName != null
                ? new FormatterArrayFieldGenerator(field, formatterName, processedFields, allFields)
                : null;
        }

        private FormatterArrayFieldGenerator(
            FieldInfoAccessor field,
            string formatterName,
            HashSet<string> processedFields,
            HashSet<string> allFields)
        {
            this.field = field;
            this.formatterName = formatterName;
            sizeFieldAttribute = field.GetCustomAttribute<ArraySizeField>();
            if (sizeFieldAttribute != null)
            {
                Validators.ValidateArraySizeField(sizeFieldAttribute, processedFields, allFields);
            }
        }

        public string GenerateEncode()
        {
            var size = $"source.{sizeFieldAttribute.Field}";
            return $"{formatterName}.{nameof(ArrayFormatter<bool>.WriteArray)}(source.{field.Name},{size}, 0,ref buffer);";
        }

        public string GenerateDecode()
        {
            var size = $"dest.{sizeFieldAttribute.Field}";
            return $"{formatterName}.{nameof(ArrayFormatter<bool>.ReadArray)}(ref buffer, dest.{field.Name}, {size},0);";
        }
    }
}