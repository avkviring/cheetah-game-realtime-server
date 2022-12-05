using JetBrains.Annotations;

namespace Cheetah.Matches.Realtime.Editor.Generator.Fields
{
    public class CodecFieldGenerator : FieldCodecGenerator
    {
        private readonly CodecsImporter codecsImporter;
        private readonly FieldInfoAccessor field;

        [CanBeNull]
        public static FieldCodecGenerator Create(CodecsImporter codecsImporter, FieldInfoAccessor fieldInfo)
        {
            return fieldInfo.FieldType.IsValueType
                   && !fieldInfo.FieldType.IsEnum
                ? new CodecFieldGenerator(codecsImporter, fieldInfo)
                : null;
        }

        private CodecFieldGenerator(CodecsImporter codecsImporter, FieldInfoAccessor field)
        {
            this.codecsImporter = codecsImporter;
            this.field = field;
        }

        public string GenerateEncode()
        {
            return $"{codecsImporter.GetCodecName(field.FieldType)}.Encode(in source.{field.Name}, ref buffer);";
        }

        public string GenerateDecode()
        {
            return $"{codecsImporter.GetCodecName(field.FieldType)}.Decode(ref buffer, ref dest.{field.Name});";
        }
    }
}