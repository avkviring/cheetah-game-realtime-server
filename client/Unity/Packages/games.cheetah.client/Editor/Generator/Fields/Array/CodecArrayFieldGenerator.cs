using System.Collections.Generic;
using System.Text;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Editor.Generator.Fields.Array.Exceptions;
using JetBrains.Annotations;

namespace Games.Cheetah.Client.Editor.Generator.Fields.Array
{
    public class CodecArrayFieldGenerator : FieldCodecGenerator
    {
        private readonly FieldInfoAccessor field;
        private readonly ArraySizeField arraySizeFieldAttribute;
        private readonly string codecName;

        [CanBeNull]
        public static FieldCodecGenerator Create(CodecsImporter codecsImporter, FieldInfoAccessor field, HashSet<string> processedFields,
            HashSet<string> allFields)
        {
            return field.FieldType.IsArray ? new CodecArrayFieldGenerator(codecsImporter, field, processedFields, allFields) : null;
        }

        private CodecArrayFieldGenerator(CodecsImporter codecsImporter, FieldInfoAccessor field, HashSet<string> processedFields,
            HashSet<string> allFields)
        {
            this.field = field;
            arraySizeFieldAttribute = field.GetCustomAttribute<ArraySizeField>() ??
                                      throw new MissingArraySizeFieldAttributeException();
            Validators.ValidateArraySizeField(arraySizeFieldAttribute, processedFields, allFields);
            codecName = codecsImporter.GetCodecName(field.FieldType.GetElementType());
        }


        public string GenerateEncode()
        {
            var result = new StringBuilder();
            var size = $"source.{arraySizeFieldAttribute.Field}";
            result.AppendLine($"for(var i=0;i<{size};i++) {{");
            result.AppendLine($"\t{codecName}.Encode(in source.{field.Name}[i],ref buffer);");
            result.AppendLine("}");
            return result.ToString();
        }

        public string GenerateDecode()
        {
            var result = new StringBuilder();
            var size = $"dest.{arraySizeFieldAttribute.Field}";
            result.AppendLine($"for(var i=0;i<{size};i++) {{");
            result.AppendLine($"\t{codecName}.Decode(ref buffer, ref dest.{field.Name}[i]);");
            result.AppendLine("}");
            return result.ToString();
        }
    }
}