using System.Collections.Generic;
using System.Runtime.CompilerServices;
using System.Text;
using Cheetah.Matches.Relay.Codec;
using Cheetah.Matches.Relay.Editor.Generator.Fields.Array.Exceptions;
using JetBrains.Annotations;

namespace Cheetah.Matches.Relay.Editor.Generator.Fields.Array
{
    public class FixedArrayFieldGenerator : FieldCodecGenerator
    {
        private readonly FixedBufferAttribute fixedBufferAttribute;
        private readonly FieldInfoAccessor field;
        private readonly ArraySizeField sizeFieldAttribute;
        private readonly string formatterName;

        [CanBeNull]
        public static FieldCodecGenerator Create(Formatters formatters, FieldInfoAccessor field, HashSet<string> processedFields,
            HashSet<string> allFields)
        {
            var fixedBufferAttribute = field.GetCustomAttribute<FixedBufferAttribute>();
            return fixedBufferAttribute != null
                ? new FixedArrayFieldGenerator(formatters, field, fixedBufferAttribute, processedFields, allFields)
                : null;
        }

        private FixedArrayFieldGenerator(Formatters formatters, FieldInfoAccessor field, FixedBufferAttribute fixedBufferAttribute,
            HashSet<string> processedFields,
            HashSet<string> allFields)
        {
            this.fixedBufferAttribute = fixedBufferAttribute;
            this.field = field;
            var elementType = fixedBufferAttribute.ElementType;
            var variableSizeCodecAttribute = field.GetCustomAttribute<VariableSizeCodec>();

            if (variableSizeCodecAttribute == null)
            {
                if (!formatters.IsSupportFixedArray(elementType))
                {
                    throw new FixedArrayUnsupportedTypeException(elementType.Name);
                }

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

            sizeFieldAttribute = field.GetCustomAttribute<ArraySizeField>();
            if (sizeFieldAttribute != null)
            {
                Validators.ValidateArraySizeField(sizeFieldAttribute, processedFields, allFields);
            }
        }

        public string GenerateEncode()
        {
            var result = new StringBuilder();
            result.AppendLine("unsafe {");
            result.AppendLine($"\tfixed ({fixedBufferAttribute.ElementType}* data = source.{field.Name}) {{");
            var size = sizeFieldAttribute != null ? $"source.{sizeFieldAttribute.Field}" : fixedBufferAttribute.Length.ToString();
            var encode = $"{formatterName}.{nameof(UnmanagedFormatter<bool>.WriteFixedArray)}(data,{size},0,ref buffer);";
            result.AppendLine("\t\t" + encode);
            result.AppendLine("\t}");
            result.AppendLine("}");
            return result.ToString();
        }

        public string GenerateDecode()
        {
            var result = new StringBuilder();
            result.AppendLine("unsafe {");
            result.AppendLine($"\tfixed ({fixedBufferAttribute.ElementType}* data = dest.{field.Name}) {{");

            var size = sizeFieldAttribute != null ? $"dest.{sizeFieldAttribute.Field}" : fixedBufferAttribute.Length.ToString();
            var decode = $"{formatterName}.{nameof(UnmanagedFormatter<bool>.ReadFixedArray)}(ref buffer, data, {size},0);";
            result.AppendLine("\t\t" + decode);

            result.AppendLine("\t}");
            result.AppendLine("}");
            return result.ToString();
        }
    }
}