#nullable enable
using Cheetah.Matches.Realtime.Codec;

namespace Cheetah.Matches.Realtime.Editor.Generator.Fields
{
    /// <summary>
    /// Типы для которых определены formatter-s
    /// </summary>
    public class FormattedPresentTypesFieldGenerator : FieldCodecGenerator
    {
        private readonly FieldInfoAccessor field;
        private readonly string formatterName;


        public static FieldCodecGenerator? Create(Formatters formatters, FieldInfoAccessor field)
        {
            var formatterName = formatters.GetFormatterInstanceName(field.FieldType);
            return formatterName == null ? null : new FormattedPresentTypesFieldGenerator(field, formatterName);
        }

        private FormattedPresentTypesFieldGenerator(FieldInfoAccessor field, string formatterName)
        {
            this.field = field;
            this.formatterName = formatterName;
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