using System.Collections.Generic;
using System.Text;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.Client.Editor.Generator.Fields.Array.Exceptions;
using JetBrains.Annotations;
using Unity.Collections;
using UnityEngine;

namespace Games.Cheetah.Client.Editor.Generator.Fields.Array
{
    public class CodecNativeListFieldGenerator : FieldCodecGenerator
    {
        private readonly FieldInfoAccessor field;
        private readonly string codecName;

        public static FieldCodecGenerator Create(CodecsImporter codecsImporter, FieldInfoAccessor field)
        {
            if (!field.FieldType.IsGenericType)
            {
                return null;
            }

            return field.FieldType.GetGenericTypeDefinition() == typeof(NativeList<>)
                ? new CodecNativeListFieldGenerator(codecsImporter, field)
                : null;
        }

        private CodecNativeListFieldGenerator(CodecsImporter codecsImporter, FieldInfoAccessor field)
        {
            this.field = field;
            codecName = codecsImporter.GetCodecName(field.FieldType.GetGenericArguments()[0]);
        }


        public string GenerateEncode()
        {
            
            var result = new StringBuilder();
            result.AppendLine($"for(var i=0;i<{field.Name}.Count();i++) {{");
            result.AppendLine($"\t{codecName}.Encode(in source.{field.Name}[i],ref buffer);");
            result.AppendLine("}");
            return result.ToString();
        }

        public string GenerateDecode()
        {
            var result = new StringBuilder();
            result.AppendLine($"for(var i=0;i<10;i++) {{");
            result.AppendLine($"\t{codecName}.Decode(ref buffer, ref dest.{field.Name}[i]);");
            result.AppendLine("}");
            return result.ToString();
        }
    }
}